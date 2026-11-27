const std = @import("std");

const COM1: u16 = 0x3f8;

// when DLAB = 0
const INTERRUPT_REG: u16 = 1;
const LINE_CTRL_REG: u16 = 3;
// when DLAB = 1
const DIVISOR_LO_REG: u16 = 0;
// when DLAB = 1
const DIVISOR_HI_REG: u16 = 1;
const INTERRUPT_IDENT_REG: u16 = 2;
const MODEM_CTRL_REG: u16 = 4;
// when DLAB = 0
const DATA_REG: u16 = 0;
const LINE_STATUS_REG: u16 = 5;

const TEST_MAGIC: u8 = 0xae;

const Error = error{
    faulty_port,
};

pub fn init() !void {
    // disable all inerrupts
    outb(COM1 + INTERRUPT_REG, 0x00);
    // set DLAB (to set thge baud rate)
    outb(COM1 + LINE_CTRL_REG, 0x80);
    // set divisor to 38400 baud (we don't need higher speeds right now tbh)
    outb(COM1 + DIVISOR_LO_REG, 0x03);
    outb(COM1 + DIVISOR_HI_REG, 0x00);
    // clear DLAB, set character length to 8 bits, 1 stop bit, and no parity because it's for paranoids
    outb(COM1 + LINE_CTRL_REG, 0x03);
    outb(COM1 + INTERRUPT_IDENT_REG, 0xc7);
    outb(COM1 + MODEM_CTRL_REG, 0x0b);
    outb(COM1 + MODEM_CTRL_REG, 0x1e);
    outb(COM1 + DATA_REG, TEST_MAGIC);

    if (inb(COM1 + DATA_REG) != TEST_MAGIC) {
        return Error.faulty_port;
    }

    outb(COM1 + MODEM_CTRL_REG, 0x0f);

    kprint("Serial initialized!\n", .{});
}

// TODO rewrite this and make it more idiomatic and safe
// CONCURRENCY lock before accessing to prevent buffer corruption
pub fn kprint(comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    const out = std.fmt.bufPrint(&buf, fmt, args) catch "><print truncated><\n";
    print(out);
}

pub fn print(str: []const u8) void {
    for (str) |c| {
        while (!isTransmitEmpty()) {}
        outb(COM1 + DATA_REG, c);
    }
}

inline fn isTransmitEmpty() bool {
    return (inb(COM1 + LINE_STATUS_REG) & 0x20) != 0;
}

inline fn outb(port: u16, value: u8) void {
    asm volatile ("out %[value], %[port]"
        :
        : [value] "{al}" (value),
          [port] "{dx}" (port),
    );
}

inline fn inb(port: u16) u8 {
    return asm volatile ("in %[port], %[ret]"
        : [ret] "={al}" (-> u8),
        : [port] "{dx}" (port),
    );
}
