const builtin = @import("builtin");
const gdt = @import("gdt.zig");
const serial = @import("serial.zig");
const idt = @import("idt.zig");

pub fn init() void {
    serial.init() catch unreachable;
    gdt.load();
    idt.init();
}

pub fn hcf() noreturn {
    while (true) {
        switch (builtin.cpu.arch) {
            .x86_64 => asm volatile ("hlt"),
            else => unreachable,
        }
    }
}
