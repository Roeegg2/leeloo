const root = @import("root.zig");
const serial = @import("serial.zig");
const gdt = @import("gdt.zig");
const std = @import("std");

// NOTE marked as extern and fields are in that specific order because the `commonIsrStub` pushes according to the SysV ABI
pub const InterruptFrame = extern struct {
    // pushed by our stub:
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
    // pushed by us:
    vector: u64,
    // pushed by the CPU (or by us when the CPU doesn't):
    error_code: u64,
    // pushed by the CPU:
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
};

pub const InterruptHandler = *const fn (*InterruptFrame) void;

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

const unique_stubs = blk: {
    var table: [256]*const fn () callconv(.naked) void = undefined;
    for (&table, 0..) |*slot, vec| slot.* = makeUniqueIsrStub(@intCast(vec));
    break :blk table;
};

var handlers: [256]?InterruptHandler = @splat(null);

var idt: Idt align(16) = @splat(GateDescriptor.missing);

pub fn registerInterruptHandler(vector: u8, handler: InterruptHandler) void {
    handlers[vector] = handler;
}

pub fn init() void {
    for (&idt, unique_stubs) |*gate, stub| {
        gate.* = .init(
            @intFromPtr(stub),
            @offsetOf(gdt.Gdt, "kernel_code"),
            .interrupt,
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

    serial.kprint("IDT loaded!\n", .{});

    registerInterruptHandler(0x80, testInterruptHandler);

    // trigger test
    asm volatile ("int $0x80");
}

// do some shit and then call the actual handler
export fn commonIsrHandler(frame: *InterruptFrame) void {
    // cast back to a u8 so we never index outside of `handlers` array, even in release builds
    const vector: u8 = @intCast(frame.vector);

    if (handlers[vector]) |handler| {
        handler(frame);
    } else if (vector < 32) {
        @panic("unhandeled interrupt encountered");
    }
}

fn makeUniqueIsrStub(comptime vector: u8) *const fn () callconv(.naked) void {
    // whether the CPU already pushed an error code onto the stack or not
    const has_error_code = switch (vector) {
        8, 10, 11, 12, 13, 14, 17, 21, 29, 30 => true,
        else => false,
    };

    return struct {
        fn uniqueStub() callconv(.naked) void {
            asm volatile (
                // push a dummy error code 42 if needed
                (if (has_error_code) "" else "pushq $42\n") ++
                // push the interrupt vector number
                    \\pushq %[vec]
                // jump over to the stub
                    \\jmp commonIsrStub
                :
                // make sure the value we push is a u64 and not a sign extended value
                : [vec] "i" (@as(u64, vector)),
            );
        }
    }.uniqueStub;
}

// stage 1 handling, which just pushes the rest of the stack frame fields and calls the common handler.
// technically we could've already handled the interrupt here, but it's nicer to separate them into
// 2 stages so we don't get ugly assembly mixed in with pure Zig code
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

fn testInterruptHandler(frame: *InterruptFrame) void {
    _ = frame;
    serial.kprint("TEST!\n", .{});
}

// COMPTIME TESTS
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
