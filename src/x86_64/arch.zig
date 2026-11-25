const gdt = @import("gdt.zig");

pub fn init() void {
    gdt.load();
}
