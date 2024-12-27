import re
# 定义寄存器初始状态
registers = {
    'a0': 0, 'a1': 0, 'a2': 0, 'a3': 0, 'a4': 0, 'a5': 0, 'a6': 0, 'a7': 0,
    't0': 0, 't1': 0, 't2': 0, 't3': 0, 't4': 0, 't5': 0, 't6': 0,
}

# 定义内存 (栈)
memory = [0] * 1024  # 1KB 内存

# 将地址从 sp 偏移转换为内存索引
def sp_offset_to_index(sp_offset):
    base_sp = len(memory) - 352  # 开始时 sp 的位置
    return base_sp + sp_offset

# 打印寄存器值
def print_registers():
    for reg in sorted(registers.keys()):
        print(f"{reg}: {registers[reg]}")

# 从文件中读取汇编指令
with open('hello.asm') as file:
    instructions = file.readlines()

# 解析并执行指令
for line in instructions:
    line = line.strip()
    if not line or line.startswith('#') or line.startswith('//'):
        continue  # 忽略空行和注释行
    # 去除,
    parts = re.split(r'[ ,]+', line)
    opcode = parts[0]

    if opcode == 'addi':
        continue
        dst, src, imm = parts[1], parts[2], int(parts[3])
        registers[dst] = registers[src] + imm
    elif opcode == 'li':
        print(f"li {parts[1]} {parts[2]}")
        dst, imm = parts[1], int(parts[2])
        registers[dst] = imm
    elif opcode == 'sw':
        src, offset = parts[1], int(parts[2].replace('(', '').replace('sp)', ''))
        memory[sp_offset_to_index(offset)] = registers[src]
    elif opcode == 'lw':
        dst, offset = parts[1], int(parts[2].replace('(', '').replace('sp)', ''))
        registers[dst] = memory[sp_offset_to_index(offset)]
    elif opcode == 'add':
        dst, src1, src2 = parts[1], parts[2], parts[3]
        registers[dst] = registers[src1] + registers[src2]
    elif opcode == 'sub':
        dst, src1, src2 = parts[1], parts[2], parts[3]
        registers[dst] = registers[src1] - registers[src2]
    elif opcode == 'rem':
        dst, src1, src2 = parts[1], parts[2], parts[3]
        registers[dst] = registers[src1] % registers[src2]
    elif opcode == 'mv':
        dst, src = parts[1], parts[2]
        registers[dst] = registers[src]
    else:
        print(f"Unknown opcode: {opcode}")
        continue

# 输出最终 a0 的值
print(f"Final value of a0: {registers['a0']}")