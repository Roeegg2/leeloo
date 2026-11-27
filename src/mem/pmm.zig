const std = @import("std");
const limine = @import("limine");
const root = @import("root.zig");
const serial = @import("../x86_64/serial.zig");

const FRAME_SIZE: usize = 0x1000;

pub const Error = error {
    OutOfMemory,
    NoUsableMemory,
};

pub const BumpAllocator = struct {
    curr: root.PhysAddr,
    end: root.PhysAddr,

    const Self = @This();

    pub fn initFromLimine(memmap_entries: []*limine.MemoryMapEntry) Self {
        var max: ?*limine.MemoryMapEntry = null;

        for (memmap_entries) |entry| {
            if (entry.type != .usable)
                continue;

            if (max == null or entry.length > max.?.length)
                max = entry;
        }

        const best = max orelse @panic("No usable memory found");
        const curr = std.mem.alignForward(u64, best.base, FRAME_SIZE);
        const end = std.mem.alignBackward(u64, best.base + best.length, FRAME_SIZE);

        serial.kprint("Bump PMM with start=0x{x} and frame count={}\n", .{curr, end / FRAME_SIZE});

        return .{
            .curr = curr,
            .end = end,
        };
    }

    pub fn alloc(self: *Self) !root.PhysAddr {
        if (self.curr >= self.end)
            return Error.OutOfMemory;

        const ret = self.curr;
        self.curr += FRAME_SIZE;

        return ret;
    }

    // technically free does "nothing" on bump, but I figured it's good practice to call `free()` anyway
    pub fn free(self: *Self, addr: root.PhysAddr) !void {
        _ = self;
        _ = addr;
    }
};

pub var allocator: BumpAllocator  = undefined;
