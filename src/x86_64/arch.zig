const gdt = @import("gdt.zig");
const serial = @import("serial.zig");

pub fn init() void {
    serial.init() catch unreachable;
    gdt.load();
}
