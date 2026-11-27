const root = @import("root.zig");
const serial = @import("serial.zig");
const gdt = @import("gdt.zig");

// NOTE marked as extern and fields are in that specific order because the `commonIsrStub` pushes according to the SysV ABI
pub const InterruptFrame = extern struct {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    vector: u64,
    error_code: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
};

pub const IsrHandler = *const fn (*InterruptFrame) void;

const GateDescriptor = packed struct(u128) {
    const GateType = enum(u4) {
        interrupt = 0xE,
        trap = 0xF,
    };

    offset0: u16,
    segment_selector: u16,
    ist: u3,
    reserved: u5 = 0,
    gate_type: GateType,
    always_zero: u1 = 0,
    dpl: u2,
    present: u1,
    offset1: u16,
    offset2: u32,
    reserved1: u32 = 0,

    const missing: GateDescriptor = .{
        .offset0 = 0,
        .segment_selector = 0,
        .ist = 0,
        .gate_type = .interrupt,
        .dpl = 0,
        .present = 0,
        .offset1 = 0,
        .offset2 = 0,
    };

    fn init(
        handler: usize,
        selector: u16,
        gate_type: GateType,
        dpl: u2,
        ist: u3,
    ) GateDescriptor {
        return .{
            .offset0 = @truncate(handler),
            .segment_selector = selector,
            .ist = ist,
            .gate_type = gate_type,
            .dpl = dpl,
            .present = 1,
            .offset1 = @truncate(handler >> 16),
            .offset2 = @truncate(handler >> 32),
        };
    }
};

const Idt = [256]GateDescriptor;

const personalized_stubs = blk: {
    var table: [256]*const fn () callconv(.naked) void = undefined;
    for (&table, 0..) |*slot, vec| slot.* = makePersonalizedIsrStub(@intCast(vec));
    break :blk table;
};

var handlers: [256]?IsrHandler = @splat(null);

var idt: Idt align(16) = @splat(GateDescriptor.missing);

pub fn registerInterruptHandler(vector: u8, handler: IsrHandler) void {
    handlers[vector] = handler;
}

pub fn init() void {
    for (&idt, personalized_stubs, 0..) |*gate, stub, vec| {
        gate.* = .init(
            @intFromPtr(stub),
            @offsetOf(gdt.Gdt, "kernel_code"),
            if (vec == 0x80) .trap else .interrupt,
            0,
            0,
        );
    }

    const idtr = gdt.DescriptorTablePointer{
        .size = @sizeOf(Idt) - 1,
        .offset = @intFromPtr(&idt),
    };

    asm volatile (
        \\lidt %[idtr]
        :
        : [idtr] "*m" (idtr),
        : .{ .memory = true });

    serial.kprint("IDT loaded!\n");

    triggerTest();
}

export fn commonIsrHandler(frame: *InterruptFrame) void {
    const vector: u8 = @intCast(frame.vector);
    if (handlers[vector]) |handler| {
        handler(frame);
    } else if (vector < 32) {
        // make unhandled interrupts simply halt
        serial.kprint("Unhandeled interrupt!");
        root.hcf();
    }
}

fn makePersonalizedIsrStub(comptime vector: u8) *const fn () callconv(.naked) void {
    const has_error_code = switch (vector) {
        8, 10, 11, 12, 13, 14, 17, 21, 29, 30 => true,
        else => false,
    };

    return struct {
        fn personalizedStub() callconv(.naked) void {
            asm volatile ((if (has_error_code) "" else "pushq $0\n") ++
                    \\pushq %[vec]
                    \\jmp commonIsrStub
                :
                : [vec] "i" (vector),
            );
        }
    }.personalizedStub;
}

export fn commonIsrStub() callconv(.naked) void {
    asm volatile (
        \\pushq %rax
        \\pushq %rbx
        \\pushq %rcx
        \\pushq %rdx
        \\pushq %rsi
        \\pushq %rdi
        \\pushq %rbp
        \\pushq %r8
        \\pushq %r9
        \\pushq %r10
        \\pushq %r11
        \\pushq %r12
        \\pushq %r13
        \\pushq %r14
        \\pushq %r15
        \\movq %rsp, %rdi
        \\call commonIsrHandler
        \\popq %r15
        \\popq %r14
        \\popq %r13
        \\popq %r12
        \\popq %r11
        \\popq %r10
        \\popq %r9
        \\popq %r8
        \\popq %rbp
        \\popq %rdi
        \\popq %rsi
        \\popq %rdx
        \\popq %rcx
        \\popq %rbx
        \\popq %rax
        \\addq $16, %rsp
        \\iretq
    );
}

fn triggerTest() void {
    asm volatile ("int $0x80");
}

comptime {
    const assert = @import("std").debug.assert;

    assert(@sizeOf(GateDescriptor) == 16);
    assert(@sizeOf(Idt) == 256 * 16);
    assert(@alignOf(@TypeOf(idt)) == 16);

    assert(@sizeOf(InterruptFrame) == 22 * 8);
    assert(@offsetOf(InterruptFrame, "r15") == 0 * 8);
    assert(@offsetOf(InterruptFrame, "rax") == 14 * 8);
    assert(@offsetOf(InterruptFrame, "vector") == 15 * 8);
    assert(@offsetOf(InterruptFrame, "error_code") == 16 * 8);
    assert(@offsetOf(InterruptFrame, "rip") == 17 * 8);
    assert(@offsetOf(InterruptFrame, "ss") == 21 * 8);

    // check the GateDescriptor bitpacking
    const test_gate = GateDescriptor.init(
        0x11223344_55667788,
        0x8,
        .interrupt,
        0,
        1,
    );
    const raw: u128 = @bitCast(test_gate);
    const expected_low: u64 = 0x5566_8E01_0008_7788;
    const expected_high: u64 = 0x00000000_11223344;

    assert(@as(u64, @truncate(raw)) == expected_low);
    assert(@as(u64, @truncate(raw >> 64)) == expected_high);
}
