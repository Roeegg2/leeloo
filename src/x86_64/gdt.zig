const serial = @import("serial.zig");

/// is this segment entry valid or not
const ACCESS_P: u8 = 1 << 7;
/// privilege levels:
const ACCESS_RING_0: u8 = 0 << 5;
const ACCESS_RING_3: u8 = 3 << 5;
///  code/data if true, TSS/LDT otherwise
const ACCESS_DESCR_TYPE: u8 = 1 << 4;
/// if clear the segment is data segment, otherwise it's a code segment
const ACCESS_E: u8 = 1 << 3;
// /// for data segments: direction bit - 1 if the segment grows down, and 0 for up
// /// for code segments: sets min ring level (i.e max privilege level) in which code in this segment can be executed
// const ACCESS_CONFORMING: u8     = 1 << 2;
/// for code segments: marks whether the segment is also readable
/// for data segments: marks whether the segment is also writeable
const ACCESS_RW: u8 = 1 << 1;
/// the CPU will set this bit if the segment is accessed
const ACCESS_A: u8 = 1 << 0;
/// sets the type of the entry as TSS (system segment descriptor)
const SYSTEM_ACCESS_TYPE_TSS_AVAIL: u8 = 0x9;

/// the size `limit` is scaled by. 0 means 1 bytes (i.e no scaling), 1 means 4KiB (i.e page)
const FLAGS_GRANULARITY_4K: u8 = 1 << 3;
const FLAGS_SIZE_32: u8 = 1 << 2;
/// whether the descriptor is a 64-bit code segment or not (used for long/compatibility mode
/// support)
const FLAGS_LONG_MODE: u8 = 1 << 1;

const KERNEL_STACK_SIZE: usize = 8192;

const Tss = extern struct {
    reserved0: u32 = 0,
    rsp0: u64 align(4) = 0,
    rsp1: u64 align(4) = 0,
    rsp2: u64 align(4) = 0,
    reserved1: u64 align(4) = 0,
    ist1: u64 align(4) = 0,
    ist2: u64 align(4) = 0,
    ist3: u64 align(4) = 0,
    ist4: u64 align(4) = 0,
    ist5: u64 align(4) = 0,
    ist6: u64 align(4) = 0,
    ist7: u64 align(4) = 0,
    reserved2: u64 align(4) = 0,
    reserved3: u16 = 0,
    iomap_base: u16 = 0,
};

/// NOTE the fields are align(1) to avoid Zig from adding padding to `size`
pub const DescriptorTablePointer = extern struct {
    size: u16 align(1),
    offset: u64 align(1),
};

pub const Gdt = extern struct {
    const SegmentDescriptor = packed struct(u64) {
        limit_lo: u16 = 0,
        base_lo: u24 = 0,
        access: u8 = 0,
        limit_hi: u4 = 0,
        flags: u4 = 0,
        base_hi: u8 = 0,
    };

    const SystemSegmentDescriptor = extern struct {
        low: SegmentDescriptor = .{},
        base_upper: u32 = 0,
        reserved: u32 = 0,
    };

    null_descr: SegmentDescriptor,
    kernel_code: SegmentDescriptor,
    kernel_data: SegmentDescriptor,
    user_data: SegmentDescriptor,
    user_code: SegmentDescriptor,
    tss: SystemSegmentDescriptor,
};

var gdt: Gdt = .{ .null_descr = .{}, .kernel_code = .{
    .limit_lo = 0xffff,
    .limit_hi = 0xf,
    .access = ACCESS_P | ACCESS_RING_0 | ACCESS_DESCR_TYPE | ACCESS_E | ACCESS_RW | ACCESS_A,
    .flags = FLAGS_GRANULARITY_4K | FLAGS_LONG_MODE,
}, .kernel_data = .{
    .limit_lo = 0xffff,
    .limit_hi = 0xf,
    .access = ACCESS_P | ACCESS_RING_0 | ACCESS_DESCR_TYPE | ACCESS_RW | ACCESS_A,
    .flags = FLAGS_GRANULARITY_4K | FLAGS_SIZE_32,
}, .user_data = .{
    .limit_lo = 0xffff,
    .limit_hi = 0xf,
    .access = ACCESS_P | ACCESS_RING_3 | ACCESS_DESCR_TYPE | ACCESS_RW | ACCESS_A,
    .flags = FLAGS_GRANULARITY_4K | FLAGS_SIZE_32,
}, .user_code = .{
    .limit_lo = 0xffff,
    .limit_hi = 0xf,
    .access = ACCESS_P | ACCESS_RING_3 | ACCESS_DESCR_TYPE | ACCESS_E | ACCESS_RW | ACCESS_A,
    .flags = FLAGS_GRANULARITY_4K | FLAGS_LONG_MODE,
}, .tss = .{} };

var tss: Tss align(64) = .{
    // set the base past the configured size to disable the IObitmap for now.
    .iomap_base = @sizeOf(Tss),
};

var kernel_stack: [KERNEL_STACK_SIZE]u8 align(16) = undefined;

/// NOTE this function does `cli`
pub fn load() void {
    // the stack grows downward, so point to the end of the array.
    tss.rsp0 = @intFromPtr(&kernel_stack) + KERNEL_STACK_SIZE;

    const tss_addr = @intFromPtr(&tss);
    gdt.tss = .{
        .low = .{
            .limit_lo = @sizeOf(Tss) - 1,
            .base_lo = @truncate(tss_addr),
            .access = ACCESS_P | ACCESS_RING_0 | SYSTEM_ACCESS_TYPE_TSS_AVAIL,
            .base_hi = @truncate(tss_addr >> 24),
        },
        .base_upper = @truncate(tss_addr >> 32),
    };

    const gdtr = DescriptorTablePointer{
        .size = @sizeOf(Gdt) - 1,
        .offset = @intFromPtr(&gdt),
    };

    const kernel_code_selector: u64 = @offsetOf(Gdt, "kernel_code");
    const kernel_data_selector: u16 = @offsetOf(Gdt, "kernel_data");

    _ = asm volatile (
        \\lgdt %[gdtr]
        \\mov %[data], %%ds
        \\mov %[data], %%es
        \\mov %[data], %%fs
        \\mov %[data], %%gs
        \\mov %[data], %%ss
        \\pushq %[code]
        \\leaq 1f(%%rip), %[tmp]
        \\pushq %[tmp]
        \\lretq
        \\1:
        // ask for a scratch reg and just mark it as output so we can ignore it
        : [tmp] "=&r" (-> u64),
        : [gdtr] "*m" (gdtr),
          // easier to interface with pushq and retq rather than fight with a misaligned 16bit
          [code] "r" (kernel_code_selector),
          [data] "r" (kernel_data_selector),
        : .{
          // tell the compiler we access mem that isn't in input/output
          .memory = true,
          // tell the compiler we modify status flags (cli executed for sanity sakes)
          .cc = true,
        });

    serial.kprint("GDT loaded!\n");

    const tss_selector: u16 = @offsetOf(Gdt, "tss");
    asm volatile ("ltr %[selector]"
        :
        : [selector] "r" (tss_selector),
        : .{ .memory = true });

    serial.kprint("TSS loaded!\n");
}

// a bunch of comptime sanity asserts
comptime {
    const assert = @import("std").debug.assert;

    assert(@sizeOf(Gdt.SegmentDescriptor) == 8);
    assert(@sizeOf(Gdt.SystemSegmentDescriptor) == 16);
    assert(@sizeOf(Gdt) == 56);
    assert(@sizeOf(Tss) == 104);
    assert(@sizeOf(DescriptorTablePointer) == 10);
    assert(@offsetOf(DescriptorTablePointer, "offset") == 2);
    assert(@bitOffsetOf(Gdt.SegmentDescriptor, "access") == 40);
    assert(@bitOffsetOf(Gdt.SegmentDescriptor, "flags") == 52);
    assert(@bitOffsetOf(Gdt.SegmentDescriptor, "base_hi") == 56);
    assert(@offsetOf(Gdt, "tss") == 0x28);
    assert(@offsetOf(Tss, "rsp0") == 4);
    assert(@offsetOf(Tss, "ist1") == 36);
    assert(@offsetOf(Tss, "iomap_base") == 102);
}
