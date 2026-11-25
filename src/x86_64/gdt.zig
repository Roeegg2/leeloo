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

const KERNEL_STACK_SIZE: usize = 8192;

var kernel_stack: [KERNEL_STACK_SIZE]u8 align(16) = undefined;

const Tss = extern struct {
    reserved_0: u32 = 0,
    rsp0: u64 align(4) = 0,
    rsp1: u64 align(4) = 0,
    rsp2: u64 align(4) = 0,
    reserved_1: u64 align(4) = 0,
    ist1: u64 align(4) = 0,
    ist2: u64 align(4) = 0,
    ist3: u64 align(4) = 0,
    ist4: u64 align(4) = 0,
    ist5: u64 align(4) = 0,
    ist6: u64 align(4) = 0,
    ist7: u64 align(4) = 0,
    reserved_2: u64 align(4) = 0,
    reserved_3: u16 = 0,
    iomap_base: u16 = 0,
};

var tss: Tss align(64) = .{
    // set the base past the configured size to disable the IObitmap for now.
    .iomap_base = @sizeOf(Tss),
};

/// the size `limit` is scaled by. 0 means 1 bytes (i.e no scaling), 1 means 4KiB (i.e page)
const FLAGS_GRANULARITY_4K: u8 = 1 << 3;
const FLAGS_SIZE_32: u8 = 1 << 2;
/// whether the descriptor is a 64-bit code segment or not (used for long/compatibility mode
/// support)
const FLAGS_LONG_MODE: u8 = 1 << 1;

/// NOTE the fields are align(1) to avoid Zig from adding padding to `size`
const DescriptorTablePointer = extern struct {
    size: u16 align(1),
    offset: u64 align(1),
};

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

const Gdt = extern struct {
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

    const code_sel: u16 = @offsetOf(Gdt, "kernel_code");
    const data_sel: u16 = @offsetOf(Gdt, "kernel_data");

    const gdtr = DescriptorTablePointer{
        .size = @sizeOf(Gdt) - 1,
        .offset = @intFromPtr(&gdt),
    };

    _ = asm volatile (
        \\cli
        \\lgdt (%[gdtr])
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
        : [tmp] "=&r" (-> u64),
        : [gdtr] "r" (&gdtr),
          [code] "r" (@as(u64, code_sel)),
          [data] "r" (data_sel),
        : .{ .memory = true });

    serial.kprint("GDT loaded!\n");

    const tss_sel: u16 = @offsetOf(Gdt, "tss");
    asm volatile ("ltr %[sel]"
        :
        : [sel] "r" (tss_sel),
        : .{ .memory = true });

    serial.kprint("TSS loaded!\n");
}

// a bunch of comptime sanity asserts
comptime {
    const assert = @import("std").debug.assert;

    assert(@sizeOf(Tss) == 104);
    assert(@sizeOf(SystemSegmentDescriptor) == 16);
    assert(@sizeOf(DescriptorTablePointer) == 10);
    assert(@offsetOf(Gdt, "tss") == 0x28);
    assert(@offsetOf(Tss, "rsp0") == 4);
    assert(@offsetOf(Tss, "ist1") == 36);
    assert(@offsetOf(Tss, "iomap_base") == 102);
}
