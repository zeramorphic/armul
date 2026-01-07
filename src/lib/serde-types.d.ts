export interface LineInfo {
    value: number,
    instr?: PrettyInstr,
    comment?: string,
};

interface PrettyInstr {
    opcode_prefix: string,
    cond: string,
    opcode_suffix: string,
    args: PrettyArgument[],
};

type PrettyArgument = RegisterArgument | PsrArgument | ShiftArgument | ConstantArgument | RegisterSetArgument;

interface RegisterArgument {
    type: 'Register',
    register: number,
    negative: boolean,
    write_back: boolean,
};

interface PsrArgument {
    type: 'Psr',
    psr: string,
    flag: boolean,
};

interface ShiftArgument {
    type: 'Shift',
    shift_type: ShiftType,
    shift_amount: ShiftAmount,
};

type ShiftType = 'LogicalLeft' | 'LogicalRight' | 'ArithmeticRight' | 'RotateRight' | 'RotateRightExtended';

type ShiftAmount = ConstantShiftAmount | RegisterShiftAmount;

interface ConstantShiftAmount {
    type: 'Constant',
    value: number,
}

interface RegisterShiftAmount {
    type: 'Register',
    value: number,
}

interface ConstantArgument {
    type: 'Constant',
    value: number,
    style: ConstantStyle,
};

type ConstantStyle = 'Address' | 'UnsignedDecimal' | 'Unknown';

interface RegisterSetArgument {
    type: 'RegisterSet',
    registers: number[],
    caret: boolean,
};

interface Registers {
    regs: number[],
}
