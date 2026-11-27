set pagination off
set confirm off
set architecture i386:x86-64
file zig-out/bin/kernel

python
import gdb, time
for _ in range(150):
    try:
        gdb.execute("target remote :1234")
        break
    except gdb.error:
        time.sleep(0.2)
end

hbreak kmain
rbreak panic

define regs
  monitor info registers
end
define tlb
  monitor info tlb
end
define mem
  monitor info mem
end
define pf
  p/x $cr2
end

continue
