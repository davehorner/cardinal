#![allow(clippy::module_inception)]
pub mod cardinal_orcas_symbols {
    #![allow(non_upper_case_globals)]
    // Auto-generated: label address & size constants
    pub const _cSYSTEM_VECTOR: usize = 0x0000;
    pub const _cSYSTEM_VECTOR_SIZE: usize = 0x0002;
    pub const _cSYSTEM_EXPANSION: usize = 0x0002;
    pub const _cSYSTEM_EXPANSION_SIZE: usize = 0x0002;
    pub const _cSYSTEM_WST: usize = 0x0004;
    pub const _cSYSTEM_WST_SIZE: usize = 0x0001;
    pub const _cSYSTEM_RST: usize = 0x0005;
    pub const _cSYSTEM_RST_SIZE: usize = 0x0001;
    pub const _cSYSTEM_METADATA: usize = 0x0006;
    pub const _cSYSTEM_METADATA_SIZE: usize = 0x0002;
    pub const _cSYSTEM_R: usize = 0x0008;
    pub const _cSYSTEM_R_SIZE: usize = 0x0002;
    pub const _cSYSTEM_G: usize = 0x000A;
    pub const _cSYSTEM_G_SIZE: usize = 0x0002;
    pub const _cSYSTEM_B: usize = 0x000C;
    pub const _cSYSTEM_B_SIZE: usize = 0x0002;
    pub const _cSYSTEM_DEBUG: usize = 0x000E;
    pub const _cSYSTEM_DEBUG_SIZE: usize = 0x0001;
    pub const _cSYSTEM_STATE: usize = 0x000F;
    pub const _cSYSTEM_STATE_SIZE: usize = 0x0001;
    pub const _cCONSOLE_VECTOR: usize = 0x0010;
    pub const _cCONSOLE_VECTOR_SIZE: usize = 0x0002;
    pub const _cCONSOLE_READ: usize = 0x0012;
    pub const _cCONSOLE_READ_SIZE: usize = 0x0001;
    pub const _cCONSOLE_PAD: usize = 0x0013;
    pub const _cCONSOLE_PAD_SIZE: usize = 0x0005;
    pub const _cCONSOLE_WRITE: usize = 0x0018;
    pub const _cCONSOLE_WRITE_SIZE: usize = 0x0008;
    pub const _cSCREEN_VECTOR: usize = 0x0020;
    pub const _cSCREEN_VECTOR_SIZE: usize = 0x0002;
    pub const _cSCREEN_WIDTH: usize = 0x0022;
    pub const _cSCREEN_WIDTH_SIZE: usize = 0x0002;
    pub const _cSCREEN_HEIGHT: usize = 0x0024;
    pub const _cSCREEN_HEIGHT_SIZE: usize = 0x0002;
    pub const _cSCREEN_AUTO: usize = 0x0026;
    pub const _cSCREEN_AUTO_SIZE: usize = 0x0001;
    pub const _cSCREEN_PAD: usize = 0x0027;
    pub const _cSCREEN_PAD_SIZE: usize = 0x0001;
    pub const _cSCREEN_X: usize = 0x0028;
    pub const _cSCREEN_X_SIZE: usize = 0x0002;
    pub const _cSCREEN_Y: usize = 0x002A;
    pub const _cSCREEN_Y_SIZE: usize = 0x0002;
    pub const _cSCREEN_ADDR: usize = 0x002C;
    pub const _cSCREEN_ADDR_SIZE: usize = 0x0002;
    pub const _cSCREEN_PIXEL: usize = 0x002E;
    pub const _cSCREEN_PIXEL_SIZE: usize = 0x0001;
    pub const _cSCREEN_SPRITE: usize = 0x002F;
    pub const _cSCREEN_SPRITE_SIZE: usize = 0x0051;
    pub const _cCONTROLLER_VECTOR: usize = 0x0080;
    pub const _cCONTROLLER_VECTOR_SIZE: usize = 0x0002;
    pub const _cCONTROLLER_BUTTON: usize = 0x0082;
    pub const _cCONTROLLER_BUTTON_SIZE: usize = 0x0001;
    pub const _cCONTROLLER_KEY: usize = 0x0083;
    pub const _cCONTROLLER_KEY_SIZE: usize = 0x000D;
    pub const _cMOUSE_VECTOR: usize = 0x0090;
    pub const _cMOUSE_VECTOR_SIZE: usize = 0x0002;
    pub const _cMOUSE_X: usize = 0x0092;
    pub const _cMOUSE_X_SIZE: usize = 0x0002;
    pub const _cMOUSE_Y: usize = 0x0094;
    pub const _cMOUSE_Y_SIZE: usize = 0x0002;
    pub const _cMOUSE_STATE: usize = 0x0096;
    pub const _cMOUSE_STATE_SIZE: usize = 0x0001;
    pub const _cMOUSE_CHORD: usize = 0x0097;
    pub const _cMOUSE_CHORD_SIZE: usize = 0x0001;
    pub const _cMOUSE_PAD: usize = 0x0098;
    pub const _cMOUSE_PAD_SIZE: usize = 0x0004;
    pub const _cMOUSE_SCROLLY: usize = 0x009C;
    pub const _cMOUSE_SCROLLY_SIZE: usize = 0x0001;
    pub const _cMOUSE_SCROLLY_HB: usize = 0x009C;
    pub const _cMOUSE_SCROLLY_HB_SIZE: usize = 0x0001;
    pub const _cMOUSE_SCROLLY_LB: usize = 0x009D;
    pub const _cMOUSE_SCROLLY_LB_SIZE: usize = 0x0003;
    pub const _cFILE_VECTOR: usize = 0x00A0;
    pub const _cFILE_VECTOR_SIZE: usize = 0x0002;
    pub const _cFILE_SUCCESS: usize = 0x00A2;
    pub const _cFILE_SUCCESS_SIZE: usize = 0x0001;
    pub const _cFILE_SUCCESS_LB: usize = 0x00A3;
    pub const _cFILE_SUCCESS_LB_SIZE: usize = 0x0001;
    pub const _cFILE_STAT: usize = 0x00A4;
    pub const _cFILE_STAT_SIZE: usize = 0x0002;
    pub const _cFILE_DELETE: usize = 0x00A6;
    pub const _cFILE_DELETE_SIZE: usize = 0x0001;
    pub const _cFILE_APPEND: usize = 0x00A7;
    pub const _cFILE_APPEND_SIZE: usize = 0x0001;
    pub const _cFILE_NAME: usize = 0x00A8;
    pub const _cFILE_NAME_SIZE: usize = 0x0002;
    pub const _cFILE_LENGTH: usize = 0x00AA;
    pub const _cFILE_LENGTH_SIZE: usize = 0x0002;
    pub const _cFILE_READ: usize = 0x00AC;
    pub const _cFILE_READ_SIZE: usize = 0x0002;
    pub const _cFILE_WRITE: usize = 0x00AE;
    pub const _cFILE_WRITE_SIZE: usize = 0x0012;
    pub const _cDATETIME: usize = 0x00C0;
    pub const _cDATETIME_SIZE: usize = 0x0002;
    pub const _cDATETIME_YEAR: usize = 0x00C0;
    pub const _cDATETIME_YEAR_SIZE: usize = 0x0002;
    pub const _cDATETIME_MONTH: usize = 0x00C2;
    pub const _cDATETIME_MONTH_SIZE: usize = 0x0001;
    pub const _cDATETIME_DAY: usize = 0x00C3;
    pub const _cDATETIME_DAY_SIZE: usize = 0x0001;
    pub const _cDATETIME_HOUR: usize = 0x00C4;
    pub const _cDATETIME_HOUR_SIZE: usize = 0x0001;
    pub const _cDATETIME_MINUTE: usize = 0x00C5;
    pub const _cDATETIME_MINUTE_SIZE: usize = 0x0001;
    pub const _cDATETIME_SECOND: usize = 0x00C6;
    pub const _cDATETIME_SECOND_SIZE: usize = 0x0001;
    pub const _cDATETIME_DOTW: usize = 0x00C7;
    pub const _cDATETIME_DOTW_SIZE: usize = 0x0001;
    pub const _cDATETIME_DOTY: usize = 0x00C8;
    pub const _cDATETIME_DOTY_SIZE: usize = 0x0002;
    pub const _cDATETIME_ISDST: usize = 0x00CA;
    pub const _cDATETIME_ISDST_SIZE: usize = 0x0036;
    pub const _cTYPES_LOCK_DEFAULT: usize = 0x0010;
    pub const _cTYPES_LOCK_DEFAULT_SIZE: usize = 0x0002;
    pub const _cTYPES_LOCK_LUT: usize = 0x0012;
    pub const _cTYPES_LOCK_LUT_SIZE: usize = 0x000A;
    pub const _cTYPES_LOCK_RIGHT: usize = 0x001C;
    pub const _cTYPES_LOCK_RIGHT_SIZE: usize = 0x0001;
    pub const _cTYPES_LOCK_OUTPUT: usize = 0x001D;
    pub const _cTYPES_LOCK_OUTPUT_SIZE: usize = 0x0063;
    pub const _cTYPES_PL: usize = 0x0001;
    pub const _cTYPES_PL_SIZE: usize = 0x0007;
    pub const _cTYPES_OP: usize = 0x0008;
    pub const _cTYPES_OP_SIZE: usize = 0x0002;
    pub const _cTYPES_IO: usize = 0x000A;
    pub const _cTYPES_IO_SIZE: usize = 0x0076;
    pub const _cSTYLES_SELECTED: usize = 0x0004;
    pub const _cSTYLES_SELECTED_SIZE: usize = 0x007C;
    pub const _cROW_WIDTH: usize = 0x0080;
    pub const _cROW_WIDTH_SIZE: usize = 0x003D;
    pub const _cTIMER: usize = 0x0000;
    pub const _cTIMER_SIZE: usize = 0x0001;
    pub const _cTIMER_BEAT: usize = 0x0000;
    pub const _cTIMER_BEAT_SIZE: usize = 0x0001;
    pub const _cTIMER_SPEED: usize = 0x0001;
    pub const _cTIMER_SPEED_SIZE: usize = 0x0001;
    pub const _cTIMER_FRAME: usize = 0x0002;
    pub const _cTIMER_FRAME_SIZE: usize = 0x0001;
    pub const _cTIMER_FRAME_LB: usize = 0x0003;
    pub const _cTIMER_FRAME_LB_SIZE: usize = 0x0001;
    pub const _cHELP: usize = 0x0004;
    pub const _cHELP_SIZE: usize = 0x0001;
    pub const _cSRC_BUF: usize = 0x0005;
    pub const _cSRC_BUF_SIZE: usize = 0x003F;
    pub const _cSRC_CAP: usize = 0x0044;
    pub const _cSRC_CAP_SIZE: usize = 0x0001;
    pub const _cGRID_LENGTH: usize = 0x0045;
    pub const _cGRID_LENGTH_SIZE: usize = 0x0002;
    pub const _cGRID_X1: usize = 0x0047;
    pub const _cGRID_X1_SIZE: usize = 0x0002;
    pub const _cGRID_Y1: usize = 0x0049;
    pub const _cGRID_Y1_SIZE: usize = 0x0002;
    pub const _cGRID_X2: usize = 0x004B;
    pub const _cGRID_X2_SIZE: usize = 0x0002;
    pub const _cGRID_Y2: usize = 0x004D;
    pub const _cGRID_Y2_SIZE: usize = 0x0002;
    pub const _cGRID_SIZE: usize = 0x004F;
    pub const _cGRID_SIZE_SIZE: usize = 0x0001;
    pub const _cGRID_WIDTH: usize = 0x004F;
    pub const _cGRID_WIDTH_SIZE: usize = 0x0001;
    pub const _cGRID_HEIGHT: usize = 0x0050;
    pub const _cGRID_HEIGHT_SIZE: usize = 0x0001;
    pub const _cSELECT_FROM: usize = 0x0051;
    pub const _cSELECT_FROM_SIZE: usize = 0x0001;
    pub const _cSELECT_X1: usize = 0x0051;
    pub const _cSELECT_X1_SIZE: usize = 0x0001;
    pub const _cSELECT_Y1: usize = 0x0052;
    pub const _cSELECT_Y1_SIZE: usize = 0x0001;
    pub const _cSELECT_TO: usize = 0x0053;
    pub const _cSELECT_TO_SIZE: usize = 0x0001;
    pub const _cSELECT_X2: usize = 0x0053;
    pub const _cSELECT_X2_SIZE: usize = 0x0001;
    pub const _cSELECT_Y2: usize = 0x0054;
    pub const _cSELECT_Y2_SIZE: usize = 0x0001;
    pub const _cHEAD_POS: usize = 0x0055;
    pub const _cHEAD_POS_SIZE: usize = 0x0001;
    pub const _cHEAD_X: usize = 0x0055;
    pub const _cHEAD_X_SIZE: usize = 0x0001;
    pub const _cHEAD_Y: usize = 0x0056;
    pub const _cHEAD_Y_SIZE: usize = 0x0001;
    pub const _cHEAD_ADDR: usize = 0x0057;
    pub const _cHEAD_ADDR_SIZE: usize = 0x0002;
    pub const _cVARIABLES_BUF: usize = 0x0059;
    pub const _cVARIABLES_BUF_SIZE: usize = 0x0024;
    pub const _cVOICES_BUF: usize = 0x007D;
    pub const _cVOICES_BUF_SIZE: usize = 0x0040;
    pub const _cVOICES_CAP: usize = 0x00BD;
    pub const _cVOICES_CAP_SIZE: usize = 0x0043;
    pub const _cON_RESET: usize = 0x0100;
    pub const _cON_RESET_SIZE: usize = 0x009F;
    pub const _cMETA: usize = 0x019F;
    pub const _cMETA_SIZE: usize = 0x0048;
    pub const _cMANIFEST_DAT: usize = 0x01E7;
    pub const _cMANIFEST_DAT_SIZE: usize = 0x001C;
    pub const _c_00: usize = 0x0203;
    pub const _c_00_SIZE: usize = 0x0028;
    pub const _c_01: usize = 0x022B;
    pub const _c_01_SIZE: usize = 0x001C;
    pub const _c_02: usize = 0x0247;
    pub const _c_02_SIZE: usize = 0x000A;
    pub const _c_03: usize = 0x0251;
    pub const _c_03_SIZE: usize = 0x0010;
    pub const _c_04: usize = 0x0261;
    pub const _c_04_SIZE: usize = 0x0001;
    pub const _cMANIFEST_SCAN: usize = 0x0262;
    pub const _cMANIFEST_SCAN_SIZE: usize = 0x0009;
    pub const _c_05: usize = 0x026B;
    pub const _c_05_SIZE: usize = 0x0006;
    pub const _cMANIFEST__CAT: usize = 0x0271;
    pub const _cMANIFEST__CAT_SIZE: usize = 0x0006;
    pub const _cMANIFEST__OPT: usize = 0x0277;
    pub const _cMANIFEST__OPT_SIZE: usize = 0x0002;
    pub const _cMANIFEST_BK: usize = 0x0279;
    pub const _cMANIFEST_BK_SIZE: usize = 0x000C;
    pub const _c_06: usize = 0x0285;
    pub const _c_06_SIZE: usize = 0x0014;
    pub const _cSRC_ON_CONSOLE: usize = 0x0299;
    pub const _cSRC_ON_CONSOLE_SIZE: usize = 0x000F;
    pub const _c_07: usize = 0x02A8;
    pub const _c_07_SIZE: usize = 0x0004;
    pub const _cSRC__INIT_: usize = 0x02AC;
    pub const _cSRC__INIT__SIZE: usize = 0x0005;
    pub const _cSRC__RESET_: usize = 0x02B1;
    pub const _cSRC__RESET__SIZE: usize = 0x0007;
    pub const _cSRC__LR: usize = 0x02B8;
    pub const _cSRC__LR_SIZE: usize = 0x000F;
    pub const _c_08: usize = 0x02C7;
    pub const _c_08_SIZE: usize = 0x0007;
    pub const _cSRC__PUSH_: usize = 0x02CE;
    pub const _cSRC__PUSH__SIZE: usize = 0x0002;
    pub const _cSRC_PTR: usize = 0x02D0;
    pub const _cSRC_PTR_SIZE: usize = 0x0007;
    pub const _cSRC__UNCHANGE_: usize = 0x02D7;
    pub const _cSRC__UNCHANGE__SIZE: usize = 0x0005;
    pub const _cSRC__CHANGE_: usize = 0x02DC;
    pub const _cSRC__CHANGE__SIZE: usize = 0x0002;
    pub const _cSRC__SET_CHANGE_: usize = 0x02DE;
    pub const _cSRC__SET_CHANGE__SIZE: usize = 0x0002;
    pub const _cSRC_LAST: usize = 0x02E0;
    pub const _cSRC_LAST_SIZE: usize = 0x0007;
    pub const _cSRC__FORCE_CHANGE_: usize = 0x02E7;
    pub const _cSRC__FORCE_CHANGE__SIZE: usize = 0x0005;
    pub const _cSRC_X: usize = 0x02EC;
    pub const _cSRC_X_SIZE: usize = 0x0006;
    pub const _cSRC_Y: usize = 0x02F2;
    pub const _cSRC_Y_SIZE: usize = 0x0013;
    pub const _c_0A: usize = 0x0305;
    pub const _c_0A_SIZE: usize = 0x0006;
    pub const _cSRC__FILL_: usize = 0x030B;
    pub const _cSRC__FILL__SIZE: usize = 0x000C;
    pub const _cSRC__LF: usize = 0x0317;
    pub const _cSRC__LF_SIZE: usize = 0x000C;
    pub const _cSRC_DEFAULT_PATH: usize = 0x0323;
    pub const _cSRC_DEFAULT_PATH_SIZE: usize = 0x000E;
    pub const _cON_BUTTON: usize = 0x0331;
    pub const _cON_BUTTON_SIZE: usize = 0x0019;
    pub const _c_0B: usize = 0x034A;
    pub const _c_0B_SIZE: usize = 0x0003;
    pub const _cON_BUTTON_ARROW: usize = 0x034D;
    pub const _cON_BUTTON_ARROW_SIZE: usize = 0x002C;
    pub const _cON_BUTTON_ARROW_X: usize = 0x0379;
    pub const _cON_BUTTON_ARROW_X_SIZE: usize = 0x0001;
    pub const _cON_BUTTON_ARROW_Y: usize = 0x037A;
    pub const _cON_BUTTON_ARROW_Y_SIZE: usize = 0x0002;
    pub const _cON_BUTTON_ARROW_MOD: usize = 0x037C;
    pub const _cON_BUTTON_ARROW_MOD_SIZE: usize = 0x000B;
    pub const _cON_BUTTON_ARROW_VEC: usize = 0x0387;
    pub const _cON_BUTTON_ARROW_VEC_SIZE: usize = 0x0004;
    pub const _cON_MOUSE: usize = 0x038B;
    pub const _cON_MOUSE_SIZE: usize = 0x0006;
    pub const _cON_MOUSE_LAST: usize = 0x0391;
    pub const _cON_MOUSE_LAST_SIZE: usize = 0x001A;
    pub const _cON_MOUSE_DOWN: usize = 0x03AB;
    pub const _cON_MOUSE_DOWN_SIZE: usize = 0x002B;
    pub const _c_0D: usize = 0x03D6;
    pub const _c_0D_SIZE: usize = 0x000A;
    pub const _c_0E: usize = 0x03E0;
    pub const _c_0E_SIZE: usize = 0x0002;
    pub const _c_0C: usize = 0x03E2;
    pub const _c_0C_SIZE: usize = 0x0007;
    pub const _cON_MOUSE_DRAG: usize = 0x03E9;
    pub const _cON_MOUSE_DRAG_SIZE: usize = 0x0008;
    pub const _cGET_POS: usize = 0x03F1;
    pub const _cGET_POS_SIZE: usize = 0x0016;
    pub const _cTYPES__RESET_: usize = 0x0407;
    pub const _cTYPES__RESET__SIZE: usize = 0x0005;
    pub const _cTYPES__VER: usize = 0x040C;
    pub const _cTYPES__VER_SIZE: usize = 0x000C;
    pub const _cTYPES__HOR: usize = 0x0418;
    pub const _cTYPES__HOR_SIZE: usize = 0x001A;
    pub const _cVARIABLES__PULL_: usize = 0x0432;
    pub const _cVARIABLES__PULL__SIZE: usize = 0x0012;
    pub const _c_0F: usize = 0x0444;
    pub const _c_0F_SIZE: usize = 0x0011;
    pub const _cVARIABLES__COMMIT_: usize = 0x0455;
    pub const _cVARIABLES__COMMIT__SIZE: usize = 0x0012;
    pub const _c_10: usize = 0x0467;
    pub const _c_10_SIZE: usize = 0x0011;
    pub const _cVARIABLES__RESET_: usize = 0x0478;
    pub const _cVARIABLES__RESET__SIZE: usize = 0x0009;
    pub const _cVARIABLES__L: usize = 0x0481;
    pub const _cVARIABLES__L_SIZE: usize = 0x000E;
    pub const _c_STEP_: usize = 0x048F;
    pub const _c_STEP__SIZE: usize = 0x000B;
    pub const _c_STEP___VER: usize = 0x049A;
    pub const _c_STEP___VER_SIZE: usize = 0x0010;
    pub const _c_STEP___HOR: usize = 0x04AA;
    pub const _c_STEP___HOR_SIZE: usize = 0x0025;
    pub const _c_11: usize = 0x04CF;
    pub const _c_11_SIZE: usize = 0x0025;
    pub const _cGRID__FIT_: usize = 0x04F4;
    pub const _cGRID__FIT__SIZE: usize = 0x005F;
    pub const _cGRID__RESET_: usize = 0x0553;
    pub const _cGRID__RESET__SIZE: usize = 0x0005;
    pub const _cGRID__RES_VER: usize = 0x0558;
    pub const _cGRID__RES_VER_SIZE: usize = 0x000C;
    pub const _cGRID__RES_HOR: usize = 0x0564;
    pub const _cGRID__RES_HOR_SIZE: usize = 0x001A;
    pub const _cGRID__REQDRAW_: usize = 0x057E;
    pub const _cGRID__REQDRAW__SIZE: usize = 0x0005;
    pub const _cGRID__TRY_DRAW_: usize = 0x0583;
    pub const _cGRID__TRY_DRAW__SIZE: usize = 0x0001;
    pub const _cGRID_REQ: usize = 0x0584;
    pub const _cGRID_REQ_SIZE: usize = 0x0005;
    pub const _c_12: usize = 0x0589;
    pub const _c_12_SIZE: usize = 0x0021;
    pub const _cGRID__VER: usize = 0x05AA;
    pub const _cGRID__VER_SIZE: usize = 0x001F;
    pub const _cGRID__HOR: usize = 0x05C9;
    pub const _cGRID__HOR_SIZE: usize = 0x001B;
    pub const _cGRID__DRAW_CELL_: usize = 0x05E4;
    pub const _cGRID__DRAW_CELL__SIZE: usize = 0x003E;
    pub const _cGRID__DRAW_CELL_LOWERCASE_: usize = 0x0622;
    pub const _cGRID__DRAW_CELL_LOWERCASE__SIZE: usize = 0x000E;
    pub const _cGRID_HIGHLIGHT: usize = 0x0630;
    pub const _cGRID_HIGHLIGHT_SIZE: usize = 0x000B;
    pub const _cGRID__DRAW_CELL_GRID_: usize = 0x063B;
    pub const _cGRID__DRAW_CELL_GRID__SIZE: usize = 0x0015;
    pub const _c_13: usize = 0x0650;
    pub const _c_13_SIZE: usize = 0x000B;
    pub const _cGRID__DRAW_CELL_PORT_: usize = 0x065B;
    pub const _cGRID__DRAW_CELL_PORT__SIZE: usize = 0x001C;
    pub const _cGRID__DRAW_CELL_SELECTED_: usize = 0x0677;
    pub const _cGRID__DRAW_CELL_SELECTED__SIZE: usize = 0x0018;
    pub const _cHELP__TOGGLE_: usize = 0x068F;
    pub const _cHELP__TOGGLE__SIZE: usize = 0x000B;
    pub const _cHELP__DRAW_: usize = 0x069A;
    pub const _cHELP__DRAW__SIZE: usize = 0x000B;
    pub const _c_14: usize = 0x06A5;
    pub const _c_14_SIZE: usize = 0x0006;
    pub const _cHELP__L: usize = 0x06AB;
    pub const _cHELP__L_SIZE: usize = 0x0047;
    pub const _c_15: usize = 0x06F2;
    pub const _c_15_SIZE: usize = 0x0023;
    pub const _cGET_BANG: usize = 0x0715;
    pub const _cGET_BANG_SIZE: usize = 0x0038;
    pub const _cGET_BANG_BANG: usize = 0x074D;
    pub const _cGET_BANG_BANG_SIZE: usize = 0x0004;
    pub const _cVOICES_FIND: usize = 0x0751;
    pub const _cVOICES_FIND_SIZE: usize = 0x0006;
    pub const _cVOICES__LF: usize = 0x0757;
    pub const _cVOICES__LF_SIZE: usize = 0x000B;
    pub const _cVOICES_T: usize = 0x0762;
    pub const _cVOICES_T_SIZE: usize = 0x0008;
    pub const _c_17: usize = 0x076A;
    pub const _c_17_SIZE: usize = 0x000B;
    pub const _c_16: usize = 0x076A;
    pub const _c_16_SIZE: usize = 0x000B;
    pub const _cVOICES_NEXT: usize = 0x0775;
    pub const _cVOICES_NEXT_SIZE: usize = 0x0003;
    pub const _cVOICES__LN: usize = 0x0778;
    pub const _cVOICES__LN_SIZE: usize = 0x000E;
    pub const _cVOICES_END: usize = 0x0786;
    pub const _cVOICES_END_SIZE: usize = 0x0002;
    pub const _cVOICES_COUNT: usize = 0x0788;
    pub const _cVOICES_COUNT_SIZE: usize = 0x0005;
    pub const _cVOICES__LC: usize = 0x078D;
    pub const _cVOICES__LC_SIZE: usize = 0x0010;
    pub const _cVOICES__RELEASE_: usize = 0x079D;
    pub const _cVOICES__RELEASE__SIZE: usize = 0x0010;
    pub const _cVOICES__ADD_: usize = 0x07AD;
    pub const _cVOICES__ADD__SIZE: usize = 0x0013;
    pub const _c_18: usize = 0x07C0;
    pub const _c_18_SIZE: usize = 0x001C;
    pub const _cVOICES__UPDATE_: usize = 0x07DC;
    pub const _cVOICES__UPDATE__SIZE: usize = 0x0005;
    pub const _cVOICES__LU: usize = 0x07E1;
    pub const _cVOICES__LU_SIZE: usize = 0x001C;
    pub const _c_1A: usize = 0x07FD;
    pub const _c_1A_SIZE: usize = 0x0001;
    pub const _c_19: usize = 0x07FE;
    pub const _c_19_SIZE: usize = 0x000C;
    pub const _cVOICES__DRAW_: usize = 0x080A;
    pub const _cVOICES__DRAW__SIZE: usize = 0x0001;
    pub const _cVOICES_X: usize = 0x080B;
    pub const _cVOICES_X_SIZE: usize = 0x0006;
    pub const _cVOICES_Y: usize = 0x0811;
    pub const _cVOICES_Y_SIZE: usize = 0x0012;
    pub const _cVOICES__WD: usize = 0x0823;
    pub const _cVOICES__WD_SIZE: usize = 0x0015;
    pub const _cFONT__DRAW_SHORT_: usize = 0x0838;
    pub const _cFONT__DRAW_SHORT__SIZE: usize = 0x0004;
    pub const _cFONT__DRAW_BYTE_: usize = 0x083C;
    pub const _cFONT__DRAW_BYTE__SIZE: usize = 0x0007;
    pub const _cFONT__DRAW_HEX_: usize = 0x0843;
    pub const _cFONT__DRAW_HEX__SIZE: usize = 0x0011;
    pub const _cFONT__DRAW_CHAR_COLOR_: usize = 0x0854;
    pub const _cFONT__DRAW_CHAR_COLOR__SIZE: usize = 0x0003;
    pub const _cFONT__DRAW_CHAR_: usize = 0x0857;
    pub const _cFONT__DRAW_CHAR__SIZE: usize = 0x000E;
    pub const _cFONT_COLOR: usize = 0x0865;
    pub const _cFONT_COLOR_SIZE: usize = 0x0004;
    pub const _cFONT__DRAW_STR_: usize = 0x0869;
    pub const _cFONT__DRAW_STR__SIZE: usize = 0x0003;
    pub const _cFONT__WHILE: usize = 0x086C;
    pub const _cFONT__WHILE_SIZE: usize = 0x0008;
    pub const _c_1C: usize = 0x0874;
    pub const _c_1C_SIZE: usize = 0x0007;
    pub const _cPORTS_GET_RIGHT1_VAL: usize = 0x087B;
    pub const _cPORTS_GET_RIGHT1_VAL_SIZE: usize = 0x0001;
    pub const _cPORTS_GET_RIGHT_VAL: usize = 0x087C;
    pub const _cPORTS_GET_RIGHT_VAL_SIZE: usize = 0x000C;
    pub const _cPORTS_GET_LEFT1_VAL: usize = 0x0888;
    pub const _cPORTS_GET_LEFT1_VAL_SIZE: usize = 0x0004;
    pub const _cPORTS_GET_LEFT_VAL: usize = 0x088C;
    pub const _cPORTS_GET_LEFT_VAL_SIZE: usize = 0x000C;
    pub const _cPORTS_GET_LEFT1_RAW: usize = 0x0898;
    pub const _cPORTS_GET_LEFT1_RAW_SIZE: usize = 0x0004;
    pub const _cPORTS_GET_LEFT_RAW: usize = 0x089C;
    pub const _cPORTS_GET_LEFT_RAW_SIZE: usize = 0x000F;
    pub const _cPORTS_GET_RIGHT1_CASE: usize = 0x08AB;
    pub const _cPORTS_GET_RIGHT1_CASE_SIZE: usize = 0x0014;
    pub const _cPORTS_GET_RIGHT1_RAW: usize = 0x08BF;
    pub const _cPORTS_GET_RIGHT1_RAW_SIZE: usize = 0x0001;
    pub const _cPORTS_GET_RIGHT_RAW: usize = 0x08C0;
    pub const _cPORTS_GET_RIGHT_RAW_SIZE: usize = 0x000F;
    pub const _cPORTS__SET_OUTPUT_BELOW_: usize = 0x08CF;
    pub const _cPORTS__SET_OUTPUT_BELOW__SIZE: usize = 0x0004;
    pub const _cPORTS__SET_OUTPUT_: usize = 0x08D3;
    pub const _cPORTS__SET_OUTPUT__SIZE: usize = 0x000F;
    pub const _cPORTS__SET_RAW_: usize = 0x08E2;
    pub const _cPORTS__SET_RAW__SIZE: usize = 0x000F;
    pub const _cPORTS__SET_LOCK_: usize = 0x08F1;
    pub const _cPORTS__SET_LOCK__SIZE: usize = 0x000A;
    pub const _cFILE__NEW_: usize = 0x08FB;
    pub const _cFILE__NEW__SIZE: usize = 0x0014;
    pub const _cFILE__REOPEN_: usize = 0x090F;
    pub const _cFILE__REOPEN__SIZE: usize = 0x0014;
    pub const _cFILE__INJECT_: usize = 0x0923;
    pub const _cFILE__INJECT__SIZE: usize = 0x000E;
    pub const _cFILE__STREAM: usize = 0x0931;
    pub const _cFILE__STREAM_SIZE: usize = 0x000E;
    pub const _c_1D: usize = 0x093F;
    pub const _c_1D_SIZE: usize = 0x0002;
    pub const _cFILE_B: usize = 0x0941;
    pub const _cFILE_B_SIZE: usize = 0x0008;
    pub const _cFILE_ANCHOR_X: usize = 0x0949;
    pub const _cFILE_ANCHOR_X_SIZE: usize = 0x0006;
    pub const _c_1E: usize = 0x094F;
    pub const _c_1E_SIZE: usize = 0x001A;
    pub const _cFILE__SAVE_: usize = 0x0969;
    pub const _cFILE__SAVE__SIZE: usize = 0x0011;
    pub const _cFILE__VER: usize = 0x097A;
    pub const _cFILE__VER_SIZE: usize = 0x0005;
    pub const _cFILE__HOR: usize = 0x097F;
    pub const _cFILE__HOR_SIZE: usize = 0x002C;
    pub const _cFILE_LB: usize = 0x09AB;
    pub const _cFILE_LB_SIZE: usize = 0x0001;
    pub const _cFILE__EXIT_: usize = 0x09AC;
    pub const _cFILE__EXIT__SIZE: usize = 0x0005;
    pub const _cEDIT__CUT_: usize = 0x09B1;
    pub const _cEDIT__CUT__SIZE: usize = 0x0008;
    pub const _cEDIT__COPY_: usize = 0x09B9;
    pub const _cEDIT__COPY__SIZE: usize = 0x000D;
    pub const _cEDIT__VER: usize = 0x09C6;
    pub const _cEDIT__VER_SIZE: usize = 0x0008;
    pub const _cEDIT__HOR: usize = 0x09CE;
    pub const _cEDIT__HOR_SIZE: usize = 0x0029;
    pub const _cEDIT__PUSH_: usize = 0x09F7;
    pub const _cEDIT__PUSH__SIZE: usize = 0x0003;
    pub const _cEDIT_PTR: usize = 0x09FA;
    pub const _cEDIT_PTR_SIZE: usize = 0x0008;
    pub const _cEDIT__PASTE_: usize = 0x0A02;
    pub const _cEDIT__PASTE__SIZE: usize = 0x000E;
    pub const _cEDIT__LP: usize = 0x0A10;
    pub const _cEDIT__LP_SIZE: usize = 0x000A;
    pub const _c_1F: usize = 0x0A1A;
    pub const _c_1F_SIZE: usize = 0x000B;
    pub const _cEDIT_ANCHOR: usize = 0x0A25;
    pub const _cEDIT_ANCHOR_SIZE: usize = 0x0007;
    pub const _c_20: usize = 0x0A2C;
    pub const _c_20_SIZE: usize = 0x001A;
    pub const _cEDIT__ERASE_: usize = 0x0A46;
    pub const _cEDIT__ERASE__SIZE: usize = 0x0005;
    pub const _cEDIT__TOGGLE_COMMENT_: usize = 0x0A4B;
    pub const _cEDIT__TOGGLE_COMMENT__SIZE: usize = 0x0027;
    pub const _cEDIT__L: usize = 0x0A72;
    pub const _cEDIT__L_SIZE: usize = 0x0002;
    pub const _cEDIT_C: usize = 0x0A74;
    pub const _cEDIT_C_SIZE: usize = 0x0033;
    pub const _cTIMER_ON_PLAY: usize = 0x0AA7;
    pub const _cTIMER_ON_PLAY_SIZE: usize = 0x000A;
    pub const _c_21: usize = 0x0AB1;
    pub const _c_21_SIZE: usize = 0x0006;
    pub const _cTIMER_ON_PAUSE: usize = 0x0AB7;
    pub const _cTIMER_ON_PAUSE_SIZE: usize = 0x0002;
    pub const _cTIMER_F: usize = 0x0AB9;
    pub const _cTIMER_F_SIZE: usize = 0x000C;
    pub const _c_22: usize = 0x0AC5;
    pub const _c_22_SIZE: usize = 0x0001;
    pub const _cTIMER__TOGGLE_: usize = 0x0AC6;
    pub const _cTIMER__TOGGLE__SIZE: usize = 0x0011;
    pub const _c_23: usize = 0x0AD7;
    pub const _c_23_SIZE: usize = 0x0009;
    pub const _cTIMER__STEP_: usize = 0x0AE0;
    pub const _cTIMER__STEP__SIZE: usize = 0x000C;
    pub const _cTIMER__DECR_: usize = 0x0AEC;
    pub const _cTIMER__DECR__SIZE: usize = 0x0005;
    pub const _cTIMER__INCR_: usize = 0x0AF1;
    pub const _cTIMER__INCR__SIZE: usize = 0x0005;
    pub const _cTIMER__MOD_: usize = 0x0AF6;
    pub const _cTIMER__MOD__SIZE: usize = 0x0004;
    pub const _cTIMER__SET_: usize = 0x0AFA;
    pub const _cTIMER__SET__SIZE: usize = 0x0013;
    pub const _cTIMER_WIDGET__DRAW_: usize = 0x0B0D;
    pub const _cTIMER_WIDGET__DRAW__SIZE: usize = 0x0001;
    pub const _cTIMER_WIDGET_X: usize = 0x0B0E;
    pub const _cTIMER_WIDGET_X_SIZE: usize = 0x0006;
    pub const _cTIMER_WIDGET_Y: usize = 0x0B14;
    pub const _cTIMER_WIDGET_Y_SIZE: usize = 0x002A;
    pub const _cDRAW_SPEED: usize = 0x0B3E;
    pub const _cDRAW_SPEED_SIZE: usize = 0x0001;
    pub const _cDRAW_SPEED_X: usize = 0x0B3F;
    pub const _cDRAW_SPEED_X_SIZE: usize = 0x0006;
    pub const _cDRAW_SPEED_Y: usize = 0x0B45;
    pub const _cDRAW_SPEED_Y_SIZE: usize = 0x000B;
    pub const _cSELECT__MOD_: usize = 0x0B50;
    pub const _cSELECT__MOD__SIZE: usize = 0x0010;
    pub const _c_25: usize = 0x0B60;
    pub const _c_25_SIZE: usize = 0x000B;
    pub const _c_24: usize = 0x0B6B;
    pub const _c_24_SIZE: usize = 0x000E;
    pub const _c_27: usize = 0x0B79;
    pub const _c_27_SIZE: usize = 0x0008;
    pub const _c_28: usize = 0x0B81;
    pub const _c_28_SIZE: usize = 0x001B;
    pub const _c_26: usize = 0x0B9C;
    pub const _c_26_SIZE: usize = 0x0008;
    pub const _c_29: usize = 0x0BA4;
    pub const _c_29_SIZE: usize = 0x0008;
    pub const _c_2A: usize = 0x0BAC;
    pub const _c_2A_SIZE: usize = 0x0015;
    pub const _cSELECT_VALIDATE_FROM: usize = 0x0BC1;
    pub const _cSELECT_VALIDATE_FROM_SIZE: usize = 0x0014;
    pub const _cSELECT_VALIDATE_TO: usize = 0x0BD5;
    pub const _cSELECT_VALIDATE_TO_SIZE: usize = 0x0018;
    pub const _cSELECT_IS_WITHIN: usize = 0x0BED;
    pub const _cSELECT_IS_WITHIN_SIZE: usize = 0x0024;
    pub const _cSELECT_OUTSIDE: usize = 0x0C11;
    pub const _cSELECT_OUTSIDE_SIZE: usize = 0x0004;
    pub const _cSELECT__RESET_: usize = 0x0C15;
    pub const _cSELECT__RESET__SIZE: usize = 0x0006;
    pub const _cSELECT__ALL_: usize = 0x0C1B;
    pub const _cSELECT__ALL__SIZE: usize = 0x0009;
    pub const _cSELECT__FROM_: usize = 0x0C24;
    pub const _cSELECT__FROM__SIZE: usize = 0x000A;
    pub const _c_2B: usize = 0x0C2E;
    pub const _c_2B_SIZE: usize = 0x000A;
    pub const _c_2C: usize = 0x0C38;
    pub const _c_2C_SIZE: usize = 0x0004;
    pub const _cSELECT__TO_: usize = 0x0C3C;
    pub const _cSELECT__TO__SIZE: usize = 0x000A;
    pub const _c_2D: usize = 0x0C46;
    pub const _c_2D_SIZE: usize = 0x000A;
    pub const _c_2E: usize = 0x0C50;
    pub const _c_2E_SIZE: usize = 0x000A;
    pub const _c_2F: usize = 0x0C5A;
    pub const _c_2F_SIZE: usize = 0x000B;
    pub const _c_30: usize = 0x0C65;
    pub const _c_30_SIZE: usize = 0x0004;
    pub const _cSELECT__RANGE_: usize = 0x0C69;
    pub const _cSELECT__RANGE__SIZE: usize = 0x000B;
    pub const _c_31: usize = 0x0C74;
    pub const _c_31_SIZE: usize = 0x0010;
    pub const _cSELECT__FILL_: usize = 0x0C84;
    pub const _cSELECT__FILL__SIZE: usize = 0x0006;
    pub const _c_32: usize = 0x0C8A;
    pub const _c_32_SIZE: usize = 0x000D;
    pub const _c_33: usize = 0x0C97;
    pub const _c_33_SIZE: usize = 0x000A;
    pub const _cSELECT__VER: usize = 0x0CA1;
    pub const _cSELECT__VER_SIZE: usize = 0x0008;
    pub const _cSELECT__HOR: usize = 0x0CA9;
    pub const _cSELECT__HOR_SIZE: usize = 0x0001;
    pub const _cSELECT_C: usize = 0x0CAA;
    pub const _cSELECT_C_SIZE: usize = 0x0027;
    pub const _cSELECT_WIDGET__DRAW_: usize = 0x0CD1;
    pub const _cSELECT_WIDGET__DRAW__SIZE: usize = 0x0001;
    pub const _cSELECT_WIDGET_X: usize = 0x0CD2;
    pub const _cSELECT_WIDGET_X_SIZE: usize = 0x0006;
    pub const _cSELECT_WIDGET_Y: usize = 0x0CD8;
    pub const _cSELECT_WIDGET_Y_SIZE: usize = 0x0030;
    pub const _c_34: usize = 0x0D08;
    pub const _c_34_SIZE: usize = 0x0004;
    pub const _cINSERT__TOGGLE_: usize = 0x0D0C;
    pub const _cINSERT__TOGGLE__SIZE: usize = 0x001D;
    pub const _cINSERT_ON_BUTTON: usize = 0x0D29;
    pub const _cINSERT_ON_BUTTON_SIZE: usize = 0x0016;
    pub const _c_35: usize = 0x0D3F;
    pub const _c_35_SIZE: usize = 0x0008;
    pub const _cINSERT_ANCHOR: usize = 0x0D47;
    pub const _cINSERT_ANCHOR_SIZE: usize = 0x000B;
    pub const _c_36: usize = 0x0D52;
    pub const _c_36_SIZE: usize = 0x0011;
    pub const _c_37: usize = 0x0D63;
    pub const _c_37_SIZE: usize = 0x0016;
    pub const _c_38: usize = 0x0D79;
    pub const _c_38_SIZE: usize = 0x0013;
    pub const _cINSERT_END: usize = 0x0D8C;
    pub const _cINSERT_END_SIZE: usize = 0x0002;
    pub const _cCURSOR__UPDATE_: usize = 0x0D8E;
    pub const _cCURSOR__UPDATE__SIZE: usize = 0x001D;
    pub const _cCURSOR__DRAW_: usize = 0x0DAB;
    pub const _cCURSOR__DRAW__SIZE: usize = 0x0001;
    pub const _cCURSOR_X: usize = 0x0DAC;
    pub const _cCURSOR_X_SIZE: usize = 0x0006;
    pub const _cCURSOR_Y: usize = 0x0DB2;
    pub const _cCURSOR_Y_SIZE: usize = 0x0009;
    pub const _cCURSOR_SPRITE_ICN: usize = 0x0DBB;
    pub const _cCURSOR_SPRITE_ICN_SIZE: usize = 0x0010;
    pub const _cRANDOM__INIT_: usize = 0x0DCB;
    pub const _cRANDOM__INIT__SIZE: usize = 0x0038;
    pub const _cRANDOM_GENERATE: usize = 0x0E03;
    pub const _cRANDOM_GENERATE_SIZE: usize = 0x0001;
    pub const _cRANDOM_X: usize = 0x0E04;
    pub const _cRANDOM_X_SIZE: usize = 0x000D;
    pub const _cRANDOM_Y: usize = 0x0E11;
    pub const _cRANDOM_Y_SIZE: usize = 0x0011;
    pub const _cTHEME__LOAD_: usize = 0x0E22;
    pub const _cTHEME__LOAD__SIZE: usize = 0x0027;
    pub const _cTHEME_R: usize = 0x0E49;
    pub const _cTHEME_R_SIZE: usize = 0x0006;
    pub const _cTHEME_G: usize = 0x0E4F;
    pub const _cTHEME_G_SIZE: usize = 0x0006;
    pub const _cTHEME_B: usize = 0x0E55;
    pub const _cTHEME_B_SIZE: usize = 0x0006;
    pub const _cTHEME_PATH: usize = 0x0E5B;
    pub const _cTHEME_PATH_SIZE: usize = 0x0007;
    pub const _cCHRVEL: usize = 0x0E62;
    pub const _cCHRVEL_SIZE: usize = 0x0019;
    pub const _cCHRVEL_HAS_VEL: usize = 0x0E7B;
    pub const _cCHRVEL_HAS_VEL_SIZE: usize = 0x0001;
    pub const _cCHRVEL_SILENCE: usize = 0x0E7C;
    pub const _cCHRVEL_SILENCE_SIZE: usize = 0x0004;
    pub const _cLERP: usize = 0x0E80;
    pub const _cLERP_SIZE: usize = 0x000F;
    pub const _c_3A: usize = 0x0E8F;
    pub const _c_3A_SIZE: usize = 0x0009;
    pub const _cLERP_NO_BELOW: usize = 0x0E98;
    pub const _cLERP_NO_BELOW_SIZE: usize = 0x0004;
    pub const _c_PHEX_: usize = 0x0E9C;
    pub const _c_PHEX__SIZE: usize = 0x0004;
    pub const _c_PHEX__B: usize = 0x0EA0;
    pub const _c_PHEX__B_SIZE: usize = 0x0007;
    pub const _c_PHEX__C: usize = 0x0EA7;
    pub const _c_PHEX__C_SIZE: usize = 0x0012;
    pub const _cDICT: usize = 0x0EB9;
    pub const _cDICT_SIZE: usize = 0x0005;
    pub const _cDICT_ORCA: usize = 0x0EB9;
    pub const _cDICT_ORCA_SIZE: usize = 0x0005;
    pub const _cDICT_VIEW: usize = 0x0EBE;
    pub const _cDICT_VIEW_SIZE: usize = 0x0005;
    pub const _cDICT_EDIT: usize = 0x0EC3;
    pub const _cDICT_EDIT_SIZE: usize = 0x0005;
    pub const _cDICT_PLAY: usize = 0x0EC8;
    pub const _cDICT_PLAY_SIZE: usize = 0x0005;
    pub const _cDICT_SELECT: usize = 0x0ECD;
    pub const _cDICT_SELECT_SIZE: usize = 0x0007;
    pub const _cDICT_GUIDE: usize = 0x0ED4;
    pub const _cDICT_GUIDE_SIZE: usize = 0x0018;
    pub const _cOP_A: usize = 0x0EEC;
    pub const _cOP_A_SIZE: usize = 0x0017;
    pub const _cLC: usize = 0x0F03;
    pub const _cLC_SIZE: usize = 0x0008;
    pub const _c_: usize = 0x0F0B;
    pub const _c__SIZE: usize = 0x0015;
    pub const _cCASE: usize = 0x0F20;
    pub const _cCASE_SIZE: usize = 0x001D;
    pub const _cOP_B: usize = 0x0F3D;
    pub const _cOP_B_SIZE: usize = 0x0045;
    pub const _c_3B: usize = 0x0F82;
    pub const _c_3B_SIZE: usize = 0x001E;
    pub const _cOP_C: usize = 0x0FA0;
    pub const _cOP_C_SIZE: usize = 0x0063;
    pub const _cOP_D: usize = 0x1003;
    pub const _cOP_D_SIZE: usize = 0x0051;
    pub const _cOP_E: usize = 0x1054;
    pub const _cOP_E_SIZE: usize = 0x004B;
    pub const _cSELF: usize = 0x109F;
    pub const _cSELF_SIZE: usize = 0x000C;
    pub const _cCOLLIDE: usize = 0x10AB;
    pub const _cCOLLIDE_SIZE: usize = 0x001E;
    pub const _cOP_F: usize = 0x10C9;
    pub const _cOP_F_SIZE: usize = 0x003F;
    pub const _cOP_G: usize = 0x1108;
    pub const _cOP_G_SIZE: usize = 0x0059;
    pub const _c_L: usize = 0x1161;
    pub const _c_L_SIZE: usize = 0x000E;
    pub const _cSAVE: usize = 0x116F;
    pub const _cSAVE_SIZE: usize = 0x000E;
    pub const _cOP_H: usize = 0x117D;
    pub const _cOP_H_SIZE: usize = 0x0037;
    pub const _cOP_I: usize = 0x11B4;
    pub const _cOP_I_SIZE: usize = 0x0074;
    pub const _cOP_J: usize = 0x1228;
    pub const _cOP_J_SIZE: usize = 0x003D;
    pub const _c_3C: usize = 0x1265;
    pub const _c_3C_SIZE: usize = 0x0001;
    pub const _c_W: usize = 0x1266;
    pub const _c_W_SIZE: usize = 0x001B;
    pub const _cOP_K: usize = 0x1281;
    pub const _cOP_K_SIZE: usize = 0x0052;
    pub const _c_3D: usize = 0x12D3;
    pub const _c_3D_SIZE: usize = 0x000A;
    pub const _cOP_L: usize = 0x12DD;
    pub const _cOP_L_SIZE: usize = 0x0059;
    pub const _cOP_M: usize = 0x1336;
    pub const _cOP_M_SIZE: usize = 0x0055;
    pub const _cOP_N: usize = 0x138B;
    pub const _cOP_N_SIZE: usize = 0x007D;
    pub const _cOP_O: usize = 0x1408;
    pub const _cOP_O_SIZE: usize = 0x004C;
    pub const _cOP_P: usize = 0x1454;
    pub const _cOP_P_SIZE: usize = 0x0063;
    pub const _cOP_Q: usize = 0x14B7;
    pub const _cOP_Q_SIZE: usize = 0x006A;
    pub const _cLOAD: usize = 0x1521;
    pub const _cLOAD_SIZE: usize = 0x0019;
    pub const _cOP_R: usize = 0x153A;
    pub const _cOP_R_SIZE: usize = 0x0064;
    pub const _cOP_S: usize = 0x159E;
    pub const _cOP_S_SIZE: usize = 0x005E;
    pub const _cCOLLIDE_WALL: usize = 0x15FC;
    pub const _cCOLLIDE_WALL_SIZE: usize = 0x0018;
    pub const _cOP_T: usize = 0x1614;
    pub const _cOP_T_SIZE: usize = 0x005A;
    pub const _cOP_U: usize = 0x166E;
    pub const _cOP_U_SIZE: usize = 0x0065;
    pub const _cOP_V: usize = 0x16D3;
    pub const _cOP_V_SIZE: usize = 0x0065;
    pub const _cIDLE: usize = 0x1738;
    pub const _cIDLE_SIZE: usize = 0x0003;
    pub const _cOP_W: usize = 0x173B;
    pub const _cOP_W_SIZE: usize = 0x007C;
    pub const _cOP_X: usize = 0x17B7;
    pub const _cOP_X_SIZE: usize = 0x004F;
    pub const _cOP_Y: usize = 0x1806;
    pub const _cOP_Y_SIZE: usize = 0x0038;
    pub const _c_3E: usize = 0x183E;
    pub const _c_3E_SIZE: usize = 0x0019;
    pub const _cOP_Z: usize = 0x1857;
    pub const _cOP_Z_SIZE: usize = 0x006C;
    pub const _cOP_BANG: usize = 0x18C3;
    pub const _cOP_BANG_SIZE: usize = 0x0026;
    pub const _cOP_COMMENT: usize = 0x18E9;
    pub const _cOP_COMMENT_SIZE: usize = 0x0036;
    pub const _cEND: usize = 0x191F;
    pub const _cEND_SIZE: usize = 0x0003;
    pub const _cOP_MIDI: usize = 0x1922;
    pub const _cOP_MIDI_SIZE: usize = 0x0063;
    pub const _cHAS_BANG: usize = 0x1985;
    pub const _cHAS_BANG_SIZE: usize = 0x0008;
    pub const _cHAS_PITCH: usize = 0x198D;
    pub const _cHAS_PITCH_SIZE: usize = 0x000A;
    pub const _cCHN: usize = 0x1997;
    pub const _cCHN_SIZE: usize = 0x0001;
    pub const _cPITCH: usize = 0x1998;
    pub const _cPITCH_SIZE: usize = 0x0002;
    pub const _cLEN: usize = 0x199A;
    pub const _cLEN_SIZE: usize = 0x0001;
    pub const _cVEL: usize = 0x199B;
    pub const _cVEL_SIZE: usize = 0x0004;
    pub const _cOP_PITCH: usize = 0x199F;
    pub const _cOP_PITCH_SIZE: usize = 0x002A;
    pub const _cHAS_NOTE: usize = 0x19C9;
    pub const _cHAS_NOTE_SIZE: usize = 0x0009;
    pub const _cIS_BANG: usize = 0x19D2;
    pub const _cIS_BANG_SIZE: usize = 0x002D;
    pub const _cOP_BYTE: usize = 0x19FF;
    pub const _cOP_BYTE_SIZE: usize = 0x005D;
    pub const _cOP_SELF: usize = 0x1A5C;
    pub const _cOP_SELF_SIZE: usize = 0x003E;
    pub const _c_3F: usize = 0x1A9A;
    pub const _c_3F_SIZE: usize = 0x0013;
    pub const _cOP_SELF__PUSH_: usize = 0x1AAD;
    pub const _cOP_SELF__PUSH__SIZE: usize = 0x0001;
    pub const _cOP_SELF_PTR: usize = 0x1AAE;
    pub const _cOP_SELF_PTR_SIZE: usize = 0x0008;
    pub const _cOP_NULL: usize = 0x1AB6;
    pub const _cOP_NULL_SIZE: usize = 0x0002;
    pub const _cHELP_LUT: usize = 0x1AB8;
    pub const _cHELP_LUT_SIZE: usize = 0x0040;
    pub const _cMIDI_LUT: usize = 0x1AF8;
    pub const _cMIDI_LUT_SIZE: usize = 0x0048;
    pub const _cCHRB36_LUT: usize = 0x1B40;
    pub const _cCHRB36_LUT_SIZE: usize = 0x0080;
    pub const _cB36CLC: usize = 0x1BC0;
    pub const _cB36CLC_SIZE: usize = 0x0024;
    pub const _cLIBRARY_LUT: usize = 0x1BE4;
    pub const _cLIBRARY_LUT_SIZE: usize = 0x0100;
    pub const _cSELECT_ICNS: usize = 0x1CE4;
    pub const _cSELECT_ICNS_SIZE: usize = 0x0020;
    pub const _cBEAT_ICN: usize = 0x1D04;
    pub const _cBEAT_ICN_SIZE: usize = 0x0010;
    pub const _cFONT_GLYPHS: usize = 0x1D14;
    pub const _cFONT_GLYPHS_SIZE: usize = 0x0200;
    pub const _cFONT_SPACE: usize = 0x1F14;
    pub const _cFONT_SPACE_SIZE: usize = 0x00E0;
    pub const _cFONT_DOT: usize = 0x1FF4;
    pub const _cFONT_DOT_SIZE: usize = 0x0520;
    pub const _cFILL_ICN: usize = 0x2514;
    pub const _cFILL_ICN_SIZE: usize = 0x0010;
    pub const _cAPPICON: usize = 0x2524;
    pub const _cAPPICON_SIZE: usize = 0x0090;
    pub const _cTYPES_BUF: usize = 0x25B4;
    pub const _cTYPES_BUF_SIZE: usize = 0x4000;
    pub const _cGRID_BUF: usize = 0x65B4;
    pub const _cGRID_BUF_SIZE: usize = 0x4000;
    pub const _cOP_SELF_BUF: usize = 0xA5B4;
    pub const _cOP_SELF_BUF_SIZE: usize = 0x0080;
    pub const _cEDIT_BUF: usize = 0xA634;
    pub const _cEDIT_BUF_SIZE: usize = 0x0000;

    /// Returns a slice of RAM for a label by name (address, size)
    pub fn get_slice<'a>(ram: &'a [u8], label: &str) -> Option<&'a [u8]> {
        match label {
            "System/vector" => Some(&ram[_cSYSTEM_VECTOR.._cSYSTEM_VECTOR + _cSYSTEM_VECTOR_SIZE]),
            "System/expansion" => {
                Some(&ram[_cSYSTEM_EXPANSION.._cSYSTEM_EXPANSION + _cSYSTEM_EXPANSION_SIZE])
            }
            "System/wst" => Some(&ram[_cSYSTEM_WST.._cSYSTEM_WST + _cSYSTEM_WST_SIZE]),
            "System/rst" => Some(&ram[_cSYSTEM_RST.._cSYSTEM_RST + _cSYSTEM_RST_SIZE]),
            "System/metadata" => {
                Some(&ram[_cSYSTEM_METADATA.._cSYSTEM_METADATA + _cSYSTEM_METADATA_SIZE])
            }
            "System/r" => Some(&ram[_cSYSTEM_R.._cSYSTEM_R + _cSYSTEM_R_SIZE]),
            "System/g" => Some(&ram[_cSYSTEM_G.._cSYSTEM_G + _cSYSTEM_G_SIZE]),
            "System/b" => Some(&ram[_cSYSTEM_B.._cSYSTEM_B + _cSYSTEM_B_SIZE]),
            "System/debug" => Some(&ram[_cSYSTEM_DEBUG.._cSYSTEM_DEBUG + _cSYSTEM_DEBUG_SIZE]),
            "System/state" => Some(&ram[_cSYSTEM_STATE.._cSYSTEM_STATE + _cSYSTEM_STATE_SIZE]),
            "Console/vector" => {
                Some(&ram[_cCONSOLE_VECTOR.._cCONSOLE_VECTOR + _cCONSOLE_VECTOR_SIZE])
            }
            "Console/read" => Some(&ram[_cCONSOLE_READ.._cCONSOLE_READ + _cCONSOLE_READ_SIZE]),
            "Console/pad" => Some(&ram[_cCONSOLE_PAD.._cCONSOLE_PAD + _cCONSOLE_PAD_SIZE]),
            "Console/write" => Some(&ram[_cCONSOLE_WRITE.._cCONSOLE_WRITE + _cCONSOLE_WRITE_SIZE]),
            "Screen/vector" => Some(&ram[_cSCREEN_VECTOR.._cSCREEN_VECTOR + _cSCREEN_VECTOR_SIZE]),
            "Screen/width" => Some(&ram[_cSCREEN_WIDTH.._cSCREEN_WIDTH + _cSCREEN_WIDTH_SIZE]),
            "Screen/height" => Some(&ram[_cSCREEN_HEIGHT.._cSCREEN_HEIGHT + _cSCREEN_HEIGHT_SIZE]),
            "Screen/auto" => Some(&ram[_cSCREEN_AUTO.._cSCREEN_AUTO + _cSCREEN_AUTO_SIZE]),
            "Screen/pad" => Some(&ram[_cSCREEN_PAD.._cSCREEN_PAD + _cSCREEN_PAD_SIZE]),
            "Screen/x" => Some(&ram[_cSCREEN_X.._cSCREEN_X + _cSCREEN_X_SIZE]),
            "Screen/y" => Some(&ram[_cSCREEN_Y.._cSCREEN_Y + _cSCREEN_Y_SIZE]),
            "Screen/addr" => Some(&ram[_cSCREEN_ADDR.._cSCREEN_ADDR + _cSCREEN_ADDR_SIZE]),
            "Screen/pixel" => Some(&ram[_cSCREEN_PIXEL.._cSCREEN_PIXEL + _cSCREEN_PIXEL_SIZE]),
            "Screen/sprite" => Some(&ram[_cSCREEN_SPRITE.._cSCREEN_SPRITE + _cSCREEN_SPRITE_SIZE]),
            "Controller/vector" => {
                Some(&ram[_cCONTROLLER_VECTOR.._cCONTROLLER_VECTOR + _cCONTROLLER_VECTOR_SIZE])
            }
            "Controller/button" => {
                Some(&ram[_cCONTROLLER_BUTTON.._cCONTROLLER_BUTTON + _cCONTROLLER_BUTTON_SIZE])
            }
            "Controller/key" => {
                Some(&ram[_cCONTROLLER_KEY.._cCONTROLLER_KEY + _cCONTROLLER_KEY_SIZE])
            }
            "Mouse/vector" => Some(&ram[_cMOUSE_VECTOR.._cMOUSE_VECTOR + _cMOUSE_VECTOR_SIZE]),
            "Mouse/x" => Some(&ram[_cMOUSE_X.._cMOUSE_X + _cMOUSE_X_SIZE]),
            "Mouse/y" => Some(&ram[_cMOUSE_Y.._cMOUSE_Y + _cMOUSE_Y_SIZE]),
            "Mouse/state" => Some(&ram[_cMOUSE_STATE.._cMOUSE_STATE + _cMOUSE_STATE_SIZE]),
            "Mouse/chord" => Some(&ram[_cMOUSE_CHORD.._cMOUSE_CHORD + _cMOUSE_CHORD_SIZE]),
            "Mouse/pad" => Some(&ram[_cMOUSE_PAD.._cMOUSE_PAD + _cMOUSE_PAD_SIZE]),
            "Mouse/scrolly" => Some(&ram[_cMOUSE_SCROLLY.._cMOUSE_SCROLLY + _cMOUSE_SCROLLY_SIZE]),
            "Mouse/scrolly-hb" => {
                Some(&ram[_cMOUSE_SCROLLY_HB.._cMOUSE_SCROLLY_HB + _cMOUSE_SCROLLY_HB_SIZE])
            }
            "Mouse/scrolly-lb" => {
                Some(&ram[_cMOUSE_SCROLLY_LB.._cMOUSE_SCROLLY_LB + _cMOUSE_SCROLLY_LB_SIZE])
            }
            "File/vector" => Some(&ram[_cFILE_VECTOR.._cFILE_VECTOR + _cFILE_VECTOR_SIZE]),
            "File/success" => Some(&ram[_cFILE_SUCCESS.._cFILE_SUCCESS + _cFILE_SUCCESS_SIZE]),
            "File/success-lb" => {
                Some(&ram[_cFILE_SUCCESS_LB.._cFILE_SUCCESS_LB + _cFILE_SUCCESS_LB_SIZE])
            }
            "File/stat" => Some(&ram[_cFILE_STAT.._cFILE_STAT + _cFILE_STAT_SIZE]),
            "File/delete" => Some(&ram[_cFILE_DELETE.._cFILE_DELETE + _cFILE_DELETE_SIZE]),
            "File/append" => Some(&ram[_cFILE_APPEND.._cFILE_APPEND + _cFILE_APPEND_SIZE]),
            "File/name" => Some(&ram[_cFILE_NAME.._cFILE_NAME + _cFILE_NAME_SIZE]),
            "File/length" => Some(&ram[_cFILE_LENGTH.._cFILE_LENGTH + _cFILE_LENGTH_SIZE]),
            "File/read" => Some(&ram[_cFILE_READ.._cFILE_READ + _cFILE_READ_SIZE]),
            "File/write" => Some(&ram[_cFILE_WRITE.._cFILE_WRITE + _cFILE_WRITE_SIZE]),
            "DateTime" => Some(&ram[_cDATETIME.._cDATETIME + _cDATETIME_SIZE]),
            "DateTime/year" => Some(&ram[_cDATETIME_YEAR.._cDATETIME_YEAR + _cDATETIME_YEAR_SIZE]),
            "DateTime/month" => {
                Some(&ram[_cDATETIME_MONTH.._cDATETIME_MONTH + _cDATETIME_MONTH_SIZE])
            }
            "DateTime/day" => Some(&ram[_cDATETIME_DAY.._cDATETIME_DAY + _cDATETIME_DAY_SIZE]),
            "DateTime/hour" => Some(&ram[_cDATETIME_HOUR.._cDATETIME_HOUR + _cDATETIME_HOUR_SIZE]),
            "DateTime/minute" => {
                Some(&ram[_cDATETIME_MINUTE.._cDATETIME_MINUTE + _cDATETIME_MINUTE_SIZE])
            }
            "DateTime/second" => {
                Some(&ram[_cDATETIME_SECOND.._cDATETIME_SECOND + _cDATETIME_SECOND_SIZE])
            }
            "DateTime/dotw" => Some(&ram[_cDATETIME_DOTW.._cDATETIME_DOTW + _cDATETIME_DOTW_SIZE]),
            "DateTime/doty" => Some(&ram[_cDATETIME_DOTY.._cDATETIME_DOTY + _cDATETIME_DOTY_SIZE]),
            "DateTime/isdst" => {
                Some(&ram[_cDATETIME_ISDST.._cDATETIME_ISDST + _cDATETIME_ISDST_SIZE])
            }
            "Types/lock-default" => {
                Some(&ram[_cTYPES_LOCK_DEFAULT.._cTYPES_LOCK_DEFAULT + _cTYPES_LOCK_DEFAULT_SIZE])
            }
            "Types/lock-lut" => {
                Some(&ram[_cTYPES_LOCK_LUT.._cTYPES_LOCK_LUT + _cTYPES_LOCK_LUT_SIZE])
            }
            "Types/lock-right" => {
                Some(&ram[_cTYPES_LOCK_RIGHT.._cTYPES_LOCK_RIGHT + _cTYPES_LOCK_RIGHT_SIZE])
            }
            "Types/lock-output" => {
                Some(&ram[_cTYPES_LOCK_OUTPUT.._cTYPES_LOCK_OUTPUT + _cTYPES_LOCK_OUTPUT_SIZE])
            }
            "Types/pl" => Some(&ram[_cTYPES_PL.._cTYPES_PL + _cTYPES_PL_SIZE]),
            "Types/op" => Some(&ram[_cTYPES_OP.._cTYPES_OP + _cTYPES_OP_SIZE]),
            "Types/io" => Some(&ram[_cTYPES_IO.._cTYPES_IO + _cTYPES_IO_SIZE]),
            "Styles/selected" => {
                Some(&ram[_cSTYLES_SELECTED.._cSTYLES_SELECTED + _cSTYLES_SELECTED_SIZE])
            }
            "Row/width" => Some(&ram[_cROW_WIDTH.._cROW_WIDTH + _cROW_WIDTH_SIZE]),
            "timer" => Some(&ram[_cTIMER.._cTIMER + _cTIMER_SIZE]),
            "timer/beat" => Some(&ram[_cTIMER_BEAT.._cTIMER_BEAT + _cTIMER_BEAT_SIZE]),
            "timer/speed" => Some(&ram[_cTIMER_SPEED.._cTIMER_SPEED + _cTIMER_SPEED_SIZE]),
            "timer/frame" => Some(&ram[_cTIMER_FRAME.._cTIMER_FRAME + _cTIMER_FRAME_SIZE]),
            "timer/frame-lb" => {
                Some(&ram[_cTIMER_FRAME_LB.._cTIMER_FRAME_LB + _cTIMER_FRAME_LB_SIZE])
            }
            "help" => Some(&ram[_cHELP.._cHELP + _cHELP_SIZE]),
            "src/buf" => Some(&ram[_cSRC_BUF.._cSRC_BUF + _cSRC_BUF_SIZE]),
            "src/cap" => Some(&ram[_cSRC_CAP.._cSRC_CAP + _cSRC_CAP_SIZE]),
            "grid/length" => Some(&ram[_cGRID_LENGTH.._cGRID_LENGTH + _cGRID_LENGTH_SIZE]),
            "grid/x1" => Some(&ram[_cGRID_X1.._cGRID_X1 + _cGRID_X1_SIZE]),
            "grid/y1" => Some(&ram[_cGRID_Y1.._cGRID_Y1 + _cGRID_Y1_SIZE]),
            "grid/x2" => Some(&ram[_cGRID_X2.._cGRID_X2 + _cGRID_X2_SIZE]),
            "grid/y2" => Some(&ram[_cGRID_Y2.._cGRID_Y2 + _cGRID_Y2_SIZE]),
            "grid/size" => Some(&ram[_cGRID_SIZE.._cGRID_SIZE + _cGRID_SIZE_SIZE]),
            "grid/width" => Some(&ram[_cGRID_WIDTH.._cGRID_WIDTH + _cGRID_WIDTH_SIZE]),
            "grid/height" => Some(&ram[_cGRID_HEIGHT.._cGRID_HEIGHT + _cGRID_HEIGHT_SIZE]),
            "select/from" => Some(&ram[_cSELECT_FROM.._cSELECT_FROM + _cSELECT_FROM_SIZE]),
            "select/x1" => Some(&ram[_cSELECT_X1.._cSELECT_X1 + _cSELECT_X1_SIZE]),
            "select/y1" => Some(&ram[_cSELECT_Y1.._cSELECT_Y1 + _cSELECT_Y1_SIZE]),
            "select/to" => Some(&ram[_cSELECT_TO.._cSELECT_TO + _cSELECT_TO_SIZE]),
            "select/x2" => Some(&ram[_cSELECT_X2.._cSELECT_X2 + _cSELECT_X2_SIZE]),
            "select/y2" => Some(&ram[_cSELECT_Y2.._cSELECT_Y2 + _cSELECT_Y2_SIZE]),
            "head/pos" => Some(&ram[_cHEAD_POS.._cHEAD_POS + _cHEAD_POS_SIZE]),
            "head/x" => Some(&ram[_cHEAD_X.._cHEAD_X + _cHEAD_X_SIZE]),
            "head/y" => Some(&ram[_cHEAD_Y.._cHEAD_Y + _cHEAD_Y_SIZE]),
            "head/addr" => Some(&ram[_cHEAD_ADDR.._cHEAD_ADDR + _cHEAD_ADDR_SIZE]),
            "variables/buf" => Some(&ram[_cVARIABLES_BUF.._cVARIABLES_BUF + _cVARIABLES_BUF_SIZE]),
            "voices/buf" => Some(&ram[_cVOICES_BUF.._cVOICES_BUF + _cVOICES_BUF_SIZE]),
            "voices/cap" => Some(&ram[_cVOICES_CAP.._cVOICES_CAP + _cVOICES_CAP_SIZE]),
            "on-reset" => Some(&ram[_cON_RESET.._cON_RESET + _cON_RESET_SIZE]),
            "meta" => Some(&ram[_cMETA.._cMETA + _cMETA_SIZE]),
            "manifest/dat" => Some(&ram[_cMANIFEST_DAT.._cMANIFEST_DAT + _cMANIFEST_DAT_SIZE]),
            "λ00" => Some(&ram[_c_00.._c_00 + _c_00_SIZE]),
            "λ01" => Some(&ram[_c_01.._c_01 + _c_01_SIZE]),
            "λ02" => Some(&ram[_c_02.._c_02 + _c_02_SIZE]),
            "λ03" => Some(&ram[_c_03.._c_03 + _c_03_SIZE]),
            "λ04" => Some(&ram[_c_04.._c_04 + _c_04_SIZE]),
            "manifest/scan" => Some(&ram[_cMANIFEST_SCAN.._cMANIFEST_SCAN + _cMANIFEST_SCAN_SIZE]),
            "λ05" => Some(&ram[_c_05.._c_05 + _c_05_SIZE]),
            "manifest/>cat" => Some(&ram[_cMANIFEST__CAT.._cMANIFEST__CAT + _cMANIFEST__CAT_SIZE]),
            "manifest/>opt" => Some(&ram[_cMANIFEST__OPT.._cMANIFEST__OPT + _cMANIFEST__OPT_SIZE]),
            "manifest/bk" => Some(&ram[_cMANIFEST_BK.._cMANIFEST_BK + _cMANIFEST_BK_SIZE]),
            "λ06" => Some(&ram[_c_06.._c_06 + _c_06_SIZE]),
            "src/on-console" => {
                Some(&ram[_cSRC_ON_CONSOLE.._cSRC_ON_CONSOLE + _cSRC_ON_CONSOLE_SIZE])
            }
            "λ07" => Some(&ram[_c_07.._c_07 + _c_07_SIZE]),
            "src/<init>" => Some(&ram[_cSRC__INIT_.._cSRC__INIT_ + _cSRC__INIT__SIZE]),
            "src/<reset>" => Some(&ram[_cSRC__RESET_.._cSRC__RESET_ + _cSRC__RESET__SIZE]),
            "src/>lr" => Some(&ram[_cSRC__LR.._cSRC__LR + _cSRC__LR_SIZE]),
            "λ08" => Some(&ram[_c_08.._c_08 + _c_08_SIZE]),
            "src/<push>" => Some(&ram[_cSRC__PUSH_.._cSRC__PUSH_ + _cSRC__PUSH__SIZE]),
            "src/ptr" => Some(&ram[_cSRC_PTR.._cSRC_PTR + _cSRC_PTR_SIZE]),
            "src/<unchange>" => {
                Some(&ram[_cSRC__UNCHANGE_.._cSRC__UNCHANGE_ + _cSRC__UNCHANGE__SIZE])
            }
            "src/<change>" => Some(&ram[_cSRC__CHANGE_.._cSRC__CHANGE_ + _cSRC__CHANGE__SIZE]),
            "src/<set-change>" => {
                Some(&ram[_cSRC__SET_CHANGE_.._cSRC__SET_CHANGE_ + _cSRC__SET_CHANGE__SIZE])
            }
            "src/last" => Some(&ram[_cSRC_LAST.._cSRC_LAST + _cSRC_LAST_SIZE]),
            "src/<force-change>" => {
                Some(&ram[_cSRC__FORCE_CHANGE_.._cSRC__FORCE_CHANGE_ + _cSRC__FORCE_CHANGE__SIZE])
            }
            "src/x" => Some(&ram[_cSRC_X.._cSRC_X + _cSRC_X_SIZE]),
            "src/y" => Some(&ram[_cSRC_Y.._cSRC_Y + _cSRC_Y_SIZE]),
            "λ0a" => Some(&ram[_c_0A.._c_0A + _c_0A_SIZE]),
            "src/<fill>" => Some(&ram[_cSRC__FILL_.._cSRC__FILL_ + _cSRC__FILL__SIZE]),
            "src/>lf" => Some(&ram[_cSRC__LF.._cSRC__LF + _cSRC__LF_SIZE]),
            "src/default-path" => {
                Some(&ram[_cSRC_DEFAULT_PATH.._cSRC_DEFAULT_PATH + _cSRC_DEFAULT_PATH_SIZE])
            }
            "on-button" => Some(&ram[_cON_BUTTON.._cON_BUTTON + _cON_BUTTON_SIZE]),
            "λ0b" => Some(&ram[_c_0B.._c_0B + _c_0B_SIZE]),
            "on-button-arrow" => {
                Some(&ram[_cON_BUTTON_ARROW.._cON_BUTTON_ARROW + _cON_BUTTON_ARROW_SIZE])
            }
            "on-button-arrow/x" => {
                Some(&ram[_cON_BUTTON_ARROW_X.._cON_BUTTON_ARROW_X + _cON_BUTTON_ARROW_X_SIZE])
            }
            "on-button-arrow/y" => {
                Some(&ram[_cON_BUTTON_ARROW_Y.._cON_BUTTON_ARROW_Y + _cON_BUTTON_ARROW_Y_SIZE])
            }
            "on-button-arrow/mod" => Some(
                &ram[_cON_BUTTON_ARROW_MOD.._cON_BUTTON_ARROW_MOD + _cON_BUTTON_ARROW_MOD_SIZE],
            ),
            "on-button-arrow/vec" => Some(
                &ram[_cON_BUTTON_ARROW_VEC.._cON_BUTTON_ARROW_VEC + _cON_BUTTON_ARROW_VEC_SIZE],
            ),
            "on-mouse" => Some(&ram[_cON_MOUSE.._cON_MOUSE + _cON_MOUSE_SIZE]),
            "on-mouse/last" => Some(&ram[_cON_MOUSE_LAST.._cON_MOUSE_LAST + _cON_MOUSE_LAST_SIZE]),
            "on-mouse/down" => Some(&ram[_cON_MOUSE_DOWN.._cON_MOUSE_DOWN + _cON_MOUSE_DOWN_SIZE]),
            "λ0d" => Some(&ram[_c_0D.._c_0D + _c_0D_SIZE]),
            "λ0e" => Some(&ram[_c_0E.._c_0E + _c_0E_SIZE]),
            "λ0c" => Some(&ram[_c_0C.._c_0C + _c_0C_SIZE]),
            "on-mouse/drag" => Some(&ram[_cON_MOUSE_DRAG.._cON_MOUSE_DRAG + _cON_MOUSE_DRAG_SIZE]),
            "get-pos" => Some(&ram[_cGET_POS.._cGET_POS + _cGET_POS_SIZE]),
            "types/<reset>" => Some(&ram[_cTYPES__RESET_.._cTYPES__RESET_ + _cTYPES__RESET__SIZE]),
            "types/>ver" => Some(&ram[_cTYPES__VER.._cTYPES__VER + _cTYPES__VER_SIZE]),
            "types/>hor" => Some(&ram[_cTYPES__HOR.._cTYPES__HOR + _cTYPES__HOR_SIZE]),
            "variables/<pull>" => {
                Some(&ram[_cVARIABLES__PULL_.._cVARIABLES__PULL_ + _cVARIABLES__PULL__SIZE])
            }
            "λ0f" => Some(&ram[_c_0F.._c_0F + _c_0F_SIZE]),
            "variables/<commit>" => {
                Some(&ram[_cVARIABLES__COMMIT_.._cVARIABLES__COMMIT_ + _cVARIABLES__COMMIT__SIZE])
            }
            "λ10" => Some(&ram[_c_10.._c_10 + _c_10_SIZE]),
            "variables/<reset>" => {
                Some(&ram[_cVARIABLES__RESET_.._cVARIABLES__RESET_ + _cVARIABLES__RESET__SIZE])
            }
            "variables/>l" => Some(&ram[_cVARIABLES__L.._cVARIABLES__L + _cVARIABLES__L_SIZE]),
            "<step>" => Some(&ram[_c_STEP_.._c_STEP_ + _c_STEP__SIZE]),
            "<step>/>ver" => Some(&ram[_c_STEP___VER.._c_STEP___VER + _c_STEP___VER_SIZE]),
            "<step>/>hor" => Some(&ram[_c_STEP___HOR.._c_STEP___HOR + _c_STEP___HOR_SIZE]),
            "λ11" => Some(&ram[_c_11.._c_11 + _c_11_SIZE]),
            "grid/<fit>" => Some(&ram[_cGRID__FIT_.._cGRID__FIT_ + _cGRID__FIT__SIZE]),
            "grid/<reset>" => Some(&ram[_cGRID__RESET_.._cGRID__RESET_ + _cGRID__RESET__SIZE]),
            "grid/>res-ver" => Some(&ram[_cGRID__RES_VER.._cGRID__RES_VER + _cGRID__RES_VER_SIZE]),
            "grid/>res-hor" => Some(&ram[_cGRID__RES_HOR.._cGRID__RES_HOR + _cGRID__RES_HOR_SIZE]),
            "grid/<reqdraw>" => {
                Some(&ram[_cGRID__REQDRAW_.._cGRID__REQDRAW_ + _cGRID__REQDRAW__SIZE])
            }
            "grid/<try-draw>" => {
                Some(&ram[_cGRID__TRY_DRAW_.._cGRID__TRY_DRAW_ + _cGRID__TRY_DRAW__SIZE])
            }
            "grid/req" => Some(&ram[_cGRID_REQ.._cGRID_REQ + _cGRID_REQ_SIZE]),
            "λ12" => Some(&ram[_c_12.._c_12 + _c_12_SIZE]),
            "grid/>ver" => Some(&ram[_cGRID__VER.._cGRID__VER + _cGRID__VER_SIZE]),
            "grid/>hor" => Some(&ram[_cGRID__HOR.._cGRID__HOR + _cGRID__HOR_SIZE]),
            "grid/<draw-cell>" => {
                Some(&ram[_cGRID__DRAW_CELL_.._cGRID__DRAW_CELL_ + _cGRID__DRAW_CELL__SIZE])
            }
            "grid/<draw-cell-lowercase>" => Some(
                &ram[_cGRID__DRAW_CELL_LOWERCASE_
                    .._cGRID__DRAW_CELL_LOWERCASE_ + _cGRID__DRAW_CELL_LOWERCASE__SIZE],
            ),
            "grid/highlight" => {
                Some(&ram[_cGRID_HIGHLIGHT.._cGRID_HIGHLIGHT + _cGRID_HIGHLIGHT_SIZE])
            }
            "grid/<draw-cell-grid>" => Some(
                &ram[_cGRID__DRAW_CELL_GRID_
                    .._cGRID__DRAW_CELL_GRID_ + _cGRID__DRAW_CELL_GRID__SIZE],
            ),
            "λ13" => Some(&ram[_c_13.._c_13 + _c_13_SIZE]),
            "grid/<draw-cell-port>" => Some(
                &ram[_cGRID__DRAW_CELL_PORT_
                    .._cGRID__DRAW_CELL_PORT_ + _cGRID__DRAW_CELL_PORT__SIZE],
            ),
            "grid/<draw-cell-selected>" => Some(
                &ram[_cGRID__DRAW_CELL_SELECTED_
                    .._cGRID__DRAW_CELL_SELECTED_ + _cGRID__DRAW_CELL_SELECTED__SIZE],
            ),
            "help/<toggle>" => Some(&ram[_cHELP__TOGGLE_.._cHELP__TOGGLE_ + _cHELP__TOGGLE__SIZE]),
            "help/<draw>" => Some(&ram[_cHELP__DRAW_.._cHELP__DRAW_ + _cHELP__DRAW__SIZE]),
            "λ14" => Some(&ram[_c_14.._c_14 + _c_14_SIZE]),
            "help/>l" => Some(&ram[_cHELP__L.._cHELP__L + _cHELP__L_SIZE]),
            "λ15" => Some(&ram[_c_15.._c_15 + _c_15_SIZE]),
            "get-bang" => Some(&ram[_cGET_BANG.._cGET_BANG + _cGET_BANG_SIZE]),
            "get-bang/bang" => Some(&ram[_cGET_BANG_BANG.._cGET_BANG_BANG + _cGET_BANG_BANG_SIZE]),
            "voices/find" => Some(&ram[_cVOICES_FIND.._cVOICES_FIND + _cVOICES_FIND_SIZE]),
            "voices/>lf" => Some(&ram[_cVOICES__LF.._cVOICES__LF + _cVOICES__LF_SIZE]),
            "voices/t" => Some(&ram[_cVOICES_T.._cVOICES_T + _cVOICES_T_SIZE]),
            "λ17" => Some(&ram[_c_17.._c_17 + _c_17_SIZE]),
            "λ16" => Some(&ram[_c_16.._c_16 + _c_16_SIZE]),
            "voices/next" => Some(&ram[_cVOICES_NEXT.._cVOICES_NEXT + _cVOICES_NEXT_SIZE]),
            "voices/>ln" => Some(&ram[_cVOICES__LN.._cVOICES__LN + _cVOICES__LN_SIZE]),
            "voices/end" => Some(&ram[_cVOICES_END.._cVOICES_END + _cVOICES_END_SIZE]),
            "voices/count" => Some(&ram[_cVOICES_COUNT.._cVOICES_COUNT + _cVOICES_COUNT_SIZE]),
            "voices/>lc" => Some(&ram[_cVOICES__LC.._cVOICES__LC + _cVOICES__LC_SIZE]),
            "voices/<release>" => {
                Some(&ram[_cVOICES__RELEASE_.._cVOICES__RELEASE_ + _cVOICES__RELEASE__SIZE])
            }
            "voices/<add>" => Some(&ram[_cVOICES__ADD_.._cVOICES__ADD_ + _cVOICES__ADD__SIZE]),
            "λ18" => Some(&ram[_c_18.._c_18 + _c_18_SIZE]),
            "voices/<update>" => {
                Some(&ram[_cVOICES__UPDATE_.._cVOICES__UPDATE_ + _cVOICES__UPDATE__SIZE])
            }
            "voices/>lu" => Some(&ram[_cVOICES__LU.._cVOICES__LU + _cVOICES__LU_SIZE]),
            "λ1a" => Some(&ram[_c_1A.._c_1A + _c_1A_SIZE]),
            "λ19" => Some(&ram[_c_19.._c_19 + _c_19_SIZE]),
            "voices/<draw>" => Some(&ram[_cVOICES__DRAW_.._cVOICES__DRAW_ + _cVOICES__DRAW__SIZE]),
            "voices/x" => Some(&ram[_cVOICES_X.._cVOICES_X + _cVOICES_X_SIZE]),
            "voices/y" => Some(&ram[_cVOICES_Y.._cVOICES_Y + _cVOICES_Y_SIZE]),
            "voices/>wd" => Some(&ram[_cVOICES__WD.._cVOICES__WD + _cVOICES__WD_SIZE]),
            "font/<draw-short>" => {
                Some(&ram[_cFONT__DRAW_SHORT_.._cFONT__DRAW_SHORT_ + _cFONT__DRAW_SHORT__SIZE])
            }
            "font/<draw-byte>" => {
                Some(&ram[_cFONT__DRAW_BYTE_.._cFONT__DRAW_BYTE_ + _cFONT__DRAW_BYTE__SIZE])
            }
            "font/<draw-hex>" => {
                Some(&ram[_cFONT__DRAW_HEX_.._cFONT__DRAW_HEX_ + _cFONT__DRAW_HEX__SIZE])
            }
            "font/<draw-char-color>" => Some(
                &ram[_cFONT__DRAW_CHAR_COLOR_
                    .._cFONT__DRAW_CHAR_COLOR_ + _cFONT__DRAW_CHAR_COLOR__SIZE],
            ),
            "font/<draw-char>" => {
                Some(&ram[_cFONT__DRAW_CHAR_.._cFONT__DRAW_CHAR_ + _cFONT__DRAW_CHAR__SIZE])
            }
            "font/color" => Some(&ram[_cFONT_COLOR.._cFONT_COLOR + _cFONT_COLOR_SIZE]),
            "font/<draw-str>" => {
                Some(&ram[_cFONT__DRAW_STR_.._cFONT__DRAW_STR_ + _cFONT__DRAW_STR__SIZE])
            }
            "font/>while" => Some(&ram[_cFONT__WHILE.._cFONT__WHILE + _cFONT__WHILE_SIZE]),
            "λ1c" => Some(&ram[_c_1C.._c_1C + _c_1C_SIZE]),
            "ports/get-right1-val" => Some(
                &ram[_cPORTS_GET_RIGHT1_VAL.._cPORTS_GET_RIGHT1_VAL + _cPORTS_GET_RIGHT1_VAL_SIZE],
            ),
            "ports/get-right-val" => Some(
                &ram[_cPORTS_GET_RIGHT_VAL.._cPORTS_GET_RIGHT_VAL + _cPORTS_GET_RIGHT_VAL_SIZE],
            ),
            "ports/get-left1-val" => Some(
                &ram[_cPORTS_GET_LEFT1_VAL.._cPORTS_GET_LEFT1_VAL + _cPORTS_GET_LEFT1_VAL_SIZE],
            ),
            "ports/get-left-val" => {
                Some(&ram[_cPORTS_GET_LEFT_VAL.._cPORTS_GET_LEFT_VAL + _cPORTS_GET_LEFT_VAL_SIZE])
            }
            "ports/get-left1-raw" => Some(
                &ram[_cPORTS_GET_LEFT1_RAW.._cPORTS_GET_LEFT1_RAW + _cPORTS_GET_LEFT1_RAW_SIZE],
            ),
            "ports/get-left-raw" => {
                Some(&ram[_cPORTS_GET_LEFT_RAW.._cPORTS_GET_LEFT_RAW + _cPORTS_GET_LEFT_RAW_SIZE])
            }
            "ports/get-right1-case" => Some(
                &ram[_cPORTS_GET_RIGHT1_CASE
                    .._cPORTS_GET_RIGHT1_CASE + _cPORTS_GET_RIGHT1_CASE_SIZE],
            ),
            "ports/get-right1-raw" => Some(
                &ram[_cPORTS_GET_RIGHT1_RAW.._cPORTS_GET_RIGHT1_RAW + _cPORTS_GET_RIGHT1_RAW_SIZE],
            ),
            "ports/get-right-raw" => Some(
                &ram[_cPORTS_GET_RIGHT_RAW.._cPORTS_GET_RIGHT_RAW + _cPORTS_GET_RIGHT_RAW_SIZE],
            ),
            "ports/<set-output-below>" => Some(
                &ram[_cPORTS__SET_OUTPUT_BELOW_
                    .._cPORTS__SET_OUTPUT_BELOW_ + _cPORTS__SET_OUTPUT_BELOW__SIZE],
            ),
            "ports/<set-output>" => {
                Some(&ram[_cPORTS__SET_OUTPUT_.._cPORTS__SET_OUTPUT_ + _cPORTS__SET_OUTPUT__SIZE])
            }
            "ports/<set-raw>" => {
                Some(&ram[_cPORTS__SET_RAW_.._cPORTS__SET_RAW_ + _cPORTS__SET_RAW__SIZE])
            }
            "ports/<set-lock>" => {
                Some(&ram[_cPORTS__SET_LOCK_.._cPORTS__SET_LOCK_ + _cPORTS__SET_LOCK__SIZE])
            }
            "file/<new>" => Some(&ram[_cFILE__NEW_.._cFILE__NEW_ + _cFILE__NEW__SIZE]),
            "file/<reopen>" => Some(&ram[_cFILE__REOPEN_.._cFILE__REOPEN_ + _cFILE__REOPEN__SIZE]),
            "file/<inject>" => Some(&ram[_cFILE__INJECT_.._cFILE__INJECT_ + _cFILE__INJECT__SIZE]),
            "file/>stream" => Some(&ram[_cFILE__STREAM.._cFILE__STREAM + _cFILE__STREAM_SIZE]),
            "λ1d" => Some(&ram[_c_1D.._c_1D + _c_1D_SIZE]),
            "file/b" => Some(&ram[_cFILE_B.._cFILE_B + _cFILE_B_SIZE]),
            "file/anchor-x" => Some(&ram[_cFILE_ANCHOR_X.._cFILE_ANCHOR_X + _cFILE_ANCHOR_X_SIZE]),
            "λ1e" => Some(&ram[_c_1E.._c_1E + _c_1E_SIZE]),
            "file/<save>" => Some(&ram[_cFILE__SAVE_.._cFILE__SAVE_ + _cFILE__SAVE__SIZE]),
            "file/>ver" => Some(&ram[_cFILE__VER.._cFILE__VER + _cFILE__VER_SIZE]),
            "file/>hor" => Some(&ram[_cFILE__HOR.._cFILE__HOR + _cFILE__HOR_SIZE]),
            "file/lb" => Some(&ram[_cFILE_LB.._cFILE_LB + _cFILE_LB_SIZE]),
            "file/<exit>" => Some(&ram[_cFILE__EXIT_.._cFILE__EXIT_ + _cFILE__EXIT__SIZE]),
            "edit/<cut>" => Some(&ram[_cEDIT__CUT_.._cEDIT__CUT_ + _cEDIT__CUT__SIZE]),
            "edit/<copy>" => Some(&ram[_cEDIT__COPY_.._cEDIT__COPY_ + _cEDIT__COPY__SIZE]),
            "edit/>ver" => Some(&ram[_cEDIT__VER.._cEDIT__VER + _cEDIT__VER_SIZE]),
            "edit/>hor" => Some(&ram[_cEDIT__HOR.._cEDIT__HOR + _cEDIT__HOR_SIZE]),
            "edit/<push>" => Some(&ram[_cEDIT__PUSH_.._cEDIT__PUSH_ + _cEDIT__PUSH__SIZE]),
            "edit/ptr" => Some(&ram[_cEDIT_PTR.._cEDIT_PTR + _cEDIT_PTR_SIZE]),
            "edit/<paste>" => Some(&ram[_cEDIT__PASTE_.._cEDIT__PASTE_ + _cEDIT__PASTE__SIZE]),
            "edit/>lp" => Some(&ram[_cEDIT__LP.._cEDIT__LP + _cEDIT__LP_SIZE]),
            "λ1f" => Some(&ram[_c_1F.._c_1F + _c_1F_SIZE]),
            "edit/anchor" => Some(&ram[_cEDIT_ANCHOR.._cEDIT_ANCHOR + _cEDIT_ANCHOR_SIZE]),
            "λ20" => Some(&ram[_c_20.._c_20 + _c_20_SIZE]),
            "edit/<erase>" => Some(&ram[_cEDIT__ERASE_.._cEDIT__ERASE_ + _cEDIT__ERASE__SIZE]),
            "edit/<toggle-comment>" => Some(
                &ram[_cEDIT__TOGGLE_COMMENT_
                    .._cEDIT__TOGGLE_COMMENT_ + _cEDIT__TOGGLE_COMMENT__SIZE],
            ),
            "edit/>l" => Some(&ram[_cEDIT__L.._cEDIT__L + _cEDIT__L_SIZE]),
            "edit/c" => Some(&ram[_cEDIT_C.._cEDIT_C + _cEDIT_C_SIZE]),
            "timer/on-play" => Some(&ram[_cTIMER_ON_PLAY.._cTIMER_ON_PLAY + _cTIMER_ON_PLAY_SIZE]),
            "λ21" => Some(&ram[_c_21.._c_21 + _c_21_SIZE]),
            "timer/on-pause" => {
                Some(&ram[_cTIMER_ON_PAUSE.._cTIMER_ON_PAUSE + _cTIMER_ON_PAUSE_SIZE])
            }
            "timer/f" => Some(&ram[_cTIMER_F.._cTIMER_F + _cTIMER_F_SIZE]),
            "λ22" => Some(&ram[_c_22.._c_22 + _c_22_SIZE]),
            "timer/<toggle>" => {
                Some(&ram[_cTIMER__TOGGLE_.._cTIMER__TOGGLE_ + _cTIMER__TOGGLE__SIZE])
            }
            "λ23" => Some(&ram[_c_23.._c_23 + _c_23_SIZE]),
            "timer/<step>" => Some(&ram[_cTIMER__STEP_.._cTIMER__STEP_ + _cTIMER__STEP__SIZE]),
            "timer/<decr>" => Some(&ram[_cTIMER__DECR_.._cTIMER__DECR_ + _cTIMER__DECR__SIZE]),
            "timer/<incr>" => Some(&ram[_cTIMER__INCR_.._cTIMER__INCR_ + _cTIMER__INCR__SIZE]),
            "timer/<mod>" => Some(&ram[_cTIMER__MOD_.._cTIMER__MOD_ + _cTIMER__MOD__SIZE]),
            "timer/<set>" => Some(&ram[_cTIMER__SET_.._cTIMER__SET_ + _cTIMER__SET__SIZE]),
            "timer/widget/<draw>" => Some(
                &ram[_cTIMER_WIDGET__DRAW_.._cTIMER_WIDGET__DRAW_ + _cTIMER_WIDGET__DRAW__SIZE],
            ),
            "timer/widget/x" => {
                Some(&ram[_cTIMER_WIDGET_X.._cTIMER_WIDGET_X + _cTIMER_WIDGET_X_SIZE])
            }
            "timer/widget/y" => {
                Some(&ram[_cTIMER_WIDGET_Y.._cTIMER_WIDGET_Y + _cTIMER_WIDGET_Y_SIZE])
            }
            "draw-speed" => Some(&ram[_cDRAW_SPEED.._cDRAW_SPEED + _cDRAW_SPEED_SIZE]),
            "draw-speed/x" => Some(&ram[_cDRAW_SPEED_X.._cDRAW_SPEED_X + _cDRAW_SPEED_X_SIZE]),
            "draw-speed/y" => Some(&ram[_cDRAW_SPEED_Y.._cDRAW_SPEED_Y + _cDRAW_SPEED_Y_SIZE]),
            "select/<mod>" => Some(&ram[_cSELECT__MOD_.._cSELECT__MOD_ + _cSELECT__MOD__SIZE]),
            "λ25" => Some(&ram[_c_25.._c_25 + _c_25_SIZE]),
            "λ24" => Some(&ram[_c_24.._c_24 + _c_24_SIZE]),
            "λ27" => Some(&ram[_c_27.._c_27 + _c_27_SIZE]),
            "λ28" => Some(&ram[_c_28.._c_28 + _c_28_SIZE]),
            "λ26" => Some(&ram[_c_26.._c_26 + _c_26_SIZE]),
            "λ29" => Some(&ram[_c_29.._c_29 + _c_29_SIZE]),
            "λ2a" => Some(&ram[_c_2A.._c_2A + _c_2A_SIZE]),
            "select/validate-from" => Some(
                &ram[_cSELECT_VALIDATE_FROM.._cSELECT_VALIDATE_FROM + _cSELECT_VALIDATE_FROM_SIZE],
            ),
            "select/validate-to" => {
                Some(&ram[_cSELECT_VALIDATE_TO.._cSELECT_VALIDATE_TO + _cSELECT_VALIDATE_TO_SIZE])
            }
            "select/is-within" => {
                Some(&ram[_cSELECT_IS_WITHIN.._cSELECT_IS_WITHIN + _cSELECT_IS_WITHIN_SIZE])
            }
            "select/outside" => {
                Some(&ram[_cSELECT_OUTSIDE.._cSELECT_OUTSIDE + _cSELECT_OUTSIDE_SIZE])
            }
            "select/<reset>" => {
                Some(&ram[_cSELECT__RESET_.._cSELECT__RESET_ + _cSELECT__RESET__SIZE])
            }
            "select/<all>" => Some(&ram[_cSELECT__ALL_.._cSELECT__ALL_ + _cSELECT__ALL__SIZE]),
            "select/<from>" => Some(&ram[_cSELECT__FROM_.._cSELECT__FROM_ + _cSELECT__FROM__SIZE]),
            "λ2b" => Some(&ram[_c_2B.._c_2B + _c_2B_SIZE]),
            "λ2c" => Some(&ram[_c_2C.._c_2C + _c_2C_SIZE]),
            "select/<to>" => Some(&ram[_cSELECT__TO_.._cSELECT__TO_ + _cSELECT__TO__SIZE]),
            "λ2d" => Some(&ram[_c_2D.._c_2D + _c_2D_SIZE]),
            "λ2e" => Some(&ram[_c_2E.._c_2E + _c_2E_SIZE]),
            "λ2f" => Some(&ram[_c_2F.._c_2F + _c_2F_SIZE]),
            "λ30" => Some(&ram[_c_30.._c_30 + _c_30_SIZE]),
            "select/<range>" => {
                Some(&ram[_cSELECT__RANGE_.._cSELECT__RANGE_ + _cSELECT__RANGE__SIZE])
            }
            "λ31" => Some(&ram[_c_31.._c_31 + _c_31_SIZE]),
            "select/<fill>" => Some(&ram[_cSELECT__FILL_.._cSELECT__FILL_ + _cSELECT__FILL__SIZE]),
            "λ32" => Some(&ram[_c_32.._c_32 + _c_32_SIZE]),
            "λ33" => Some(&ram[_c_33.._c_33 + _c_33_SIZE]),
            "select/>ver" => Some(&ram[_cSELECT__VER.._cSELECT__VER + _cSELECT__VER_SIZE]),
            "select/>hor" => Some(&ram[_cSELECT__HOR.._cSELECT__HOR + _cSELECT__HOR_SIZE]),
            "select/c" => Some(&ram[_cSELECT_C.._cSELECT_C + _cSELECT_C_SIZE]),
            "select/widget/<draw>" => Some(
                &ram[_cSELECT_WIDGET__DRAW_.._cSELECT_WIDGET__DRAW_ + _cSELECT_WIDGET__DRAW__SIZE],
            ),
            "select/widget/x" => {
                Some(&ram[_cSELECT_WIDGET_X.._cSELECT_WIDGET_X + _cSELECT_WIDGET_X_SIZE])
            }
            "select/widget/y" => {
                Some(&ram[_cSELECT_WIDGET_Y.._cSELECT_WIDGET_Y + _cSELECT_WIDGET_Y_SIZE])
            }
            "λ34" => Some(&ram[_c_34.._c_34 + _c_34_SIZE]),
            "insert/<toggle>" => {
                Some(&ram[_cINSERT__TOGGLE_.._cINSERT__TOGGLE_ + _cINSERT__TOGGLE__SIZE])
            }
            "insert/on-button" => {
                Some(&ram[_cINSERT_ON_BUTTON.._cINSERT_ON_BUTTON + _cINSERT_ON_BUTTON_SIZE])
            }
            "λ35" => Some(&ram[_c_35.._c_35 + _c_35_SIZE]),
            "insert/anchor" => Some(&ram[_cINSERT_ANCHOR.._cINSERT_ANCHOR + _cINSERT_ANCHOR_SIZE]),
            "λ36" => Some(&ram[_c_36.._c_36 + _c_36_SIZE]),
            "λ37" => Some(&ram[_c_37.._c_37 + _c_37_SIZE]),
            "λ38" => Some(&ram[_c_38.._c_38 + _c_38_SIZE]),
            "insert/end" => Some(&ram[_cINSERT_END.._cINSERT_END + _cINSERT_END_SIZE]),
            "cursor/<update>" => {
                Some(&ram[_cCURSOR__UPDATE_.._cCURSOR__UPDATE_ + _cCURSOR__UPDATE__SIZE])
            }
            "cursor/<draw>" => Some(&ram[_cCURSOR__DRAW_.._cCURSOR__DRAW_ + _cCURSOR__DRAW__SIZE]),
            "cursor/x" => Some(&ram[_cCURSOR_X.._cCURSOR_X + _cCURSOR_X_SIZE]),
            "cursor/y" => Some(&ram[_cCURSOR_Y.._cCURSOR_Y + _cCURSOR_Y_SIZE]),
            "cursor/sprite-icn" => {
                Some(&ram[_cCURSOR_SPRITE_ICN.._cCURSOR_SPRITE_ICN + _cCURSOR_SPRITE_ICN_SIZE])
            }
            "random/<init>" => Some(&ram[_cRANDOM__INIT_.._cRANDOM__INIT_ + _cRANDOM__INIT__SIZE]),
            "random/generate" => {
                Some(&ram[_cRANDOM_GENERATE.._cRANDOM_GENERATE + _cRANDOM_GENERATE_SIZE])
            }
            "random/x" => Some(&ram[_cRANDOM_X.._cRANDOM_X + _cRANDOM_X_SIZE]),
            "random/y" => Some(&ram[_cRANDOM_Y.._cRANDOM_Y + _cRANDOM_Y_SIZE]),
            "theme/<load>" => Some(&ram[_cTHEME__LOAD_.._cTHEME__LOAD_ + _cTHEME__LOAD__SIZE]),
            "theme/r" => Some(&ram[_cTHEME_R.._cTHEME_R + _cTHEME_R_SIZE]),
            "theme/g" => Some(&ram[_cTHEME_G.._cTHEME_G + _cTHEME_G_SIZE]),
            "theme/b" => Some(&ram[_cTHEME_B.._cTHEME_B + _cTHEME_B_SIZE]),
            "theme/path" => Some(&ram[_cTHEME_PATH.._cTHEME_PATH + _cTHEME_PATH_SIZE]),
            "chrvel" => Some(&ram[_cCHRVEL.._cCHRVEL + _cCHRVEL_SIZE]),
            "chrvel/has-vel" => {
                Some(&ram[_cCHRVEL_HAS_VEL.._cCHRVEL_HAS_VEL + _cCHRVEL_HAS_VEL_SIZE])
            }
            "chrvel/silence" => {
                Some(&ram[_cCHRVEL_SILENCE.._cCHRVEL_SILENCE + _cCHRVEL_SILENCE_SIZE])
            }
            "lerp" => Some(&ram[_cLERP.._cLERP + _cLERP_SIZE]),
            "λ3a" => Some(&ram[_c_3A.._c_3A + _c_3A_SIZE]),
            "lerp/no-below" => Some(&ram[_cLERP_NO_BELOW.._cLERP_NO_BELOW + _cLERP_NO_BELOW_SIZE]),
            "<phex>" => Some(&ram[_c_PHEX_.._c_PHEX_ + _c_PHEX__SIZE]),
            "<phex>/b" => Some(&ram[_c_PHEX__B.._c_PHEX__B + _c_PHEX__B_SIZE]),
            "<phex>/c" => Some(&ram[_c_PHEX__C.._c_PHEX__C + _c_PHEX__C_SIZE]),
            "dict" => Some(&ram[_cDICT.._cDICT + _cDICT_SIZE]),
            "dict/orca" => Some(&ram[_cDICT_ORCA.._cDICT_ORCA + _cDICT_ORCA_SIZE]),
            "dict/view" => Some(&ram[_cDICT_VIEW.._cDICT_VIEW + _cDICT_VIEW_SIZE]),
            "dict/edit" => Some(&ram[_cDICT_EDIT.._cDICT_EDIT + _cDICT_EDIT_SIZE]),
            "dict/play" => Some(&ram[_cDICT_PLAY.._cDICT_PLAY + _cDICT_PLAY_SIZE]),
            "dict/select" => Some(&ram[_cDICT_SELECT.._cDICT_SELECT + _cDICT_SELECT_SIZE]),
            "dict/guide" => Some(&ram[_cDICT_GUIDE.._cDICT_GUIDE + _cDICT_GUIDE_SIZE]),
            "op-a" => Some(&ram[_cOP_A.._cOP_A + _cOP_A_SIZE]),
            "lc" => Some(&ram[_cLC.._cLC + _cLC_SIZE]),
            "*" => Some(&ram[_c_.._c_ + _c__SIZE]),
            "case" => Some(&ram[_cCASE.._cCASE + _cCASE_SIZE]),
            "op-b" => Some(&ram[_cOP_B.._cOP_B + _cOP_B_SIZE]),
            "λ3b" => Some(&ram[_c_3B.._c_3B + _c_3B_SIZE]),
            "op-c" => Some(&ram[_cOP_C.._cOP_C + _cOP_C_SIZE]),
            "op-d" => Some(&ram[_cOP_D.._cOP_D + _cOP_D_SIZE]),
            "op-e" => Some(&ram[_cOP_E.._cOP_E + _cOP_E_SIZE]),
            "self" => Some(&ram[_cSELF.._cSELF + _cSELF_SIZE]),
            "collide" => Some(&ram[_cCOLLIDE.._cCOLLIDE + _cCOLLIDE_SIZE]),
            "op-f" => Some(&ram[_cOP_F.._cOP_F + _cOP_F_SIZE]),
            "op-g" => Some(&ram[_cOP_G.._cOP_G + _cOP_G_SIZE]),
            ">l" => Some(&ram[_c_L.._c_L + _c_L_SIZE]),
            "save" => Some(&ram[_cSAVE.._cSAVE + _cSAVE_SIZE]),
            "op-h" => Some(&ram[_cOP_H.._cOP_H + _cOP_H_SIZE]),
            "op-i" => Some(&ram[_cOP_I.._cOP_I + _cOP_I_SIZE]),
            "op-j" => Some(&ram[_cOP_J.._cOP_J + _cOP_J_SIZE]),
            "λ3c" => Some(&ram[_c_3C.._c_3C + _c_3C_SIZE]),
            ">w" => Some(&ram[_c_W.._c_W + _c_W_SIZE]),
            "op-k" => Some(&ram[_cOP_K.._cOP_K + _cOP_K_SIZE]),
            "λ3d" => Some(&ram[_c_3D.._c_3D + _c_3D_SIZE]),
            "op-l" => Some(&ram[_cOP_L.._cOP_L + _cOP_L_SIZE]),
            "op-m" => Some(&ram[_cOP_M.._cOP_M + _cOP_M_SIZE]),
            "op-n" => Some(&ram[_cOP_N.._cOP_N + _cOP_N_SIZE]),
            "op-o" => Some(&ram[_cOP_O.._cOP_O + _cOP_O_SIZE]),
            "op-p" => Some(&ram[_cOP_P.._cOP_P + _cOP_P_SIZE]),
            "op-q" => Some(&ram[_cOP_Q.._cOP_Q + _cOP_Q_SIZE]),
            "load" => Some(&ram[_cLOAD.._cLOAD + _cLOAD_SIZE]),
            "op-r" => Some(&ram[_cOP_R.._cOP_R + _cOP_R_SIZE]),
            "op-s" => Some(&ram[_cOP_S.._cOP_S + _cOP_S_SIZE]),
            "collide-wall" => Some(&ram[_cCOLLIDE_WALL.._cCOLLIDE_WALL + _cCOLLIDE_WALL_SIZE]),
            "op-t" => Some(&ram[_cOP_T.._cOP_T + _cOP_T_SIZE]),
            "op-u" => Some(&ram[_cOP_U.._cOP_U + _cOP_U_SIZE]),
            "op-v" => Some(&ram[_cOP_V.._cOP_V + _cOP_V_SIZE]),
            "idle" => Some(&ram[_cIDLE.._cIDLE + _cIDLE_SIZE]),
            "op-w" => Some(&ram[_cOP_W.._cOP_W + _cOP_W_SIZE]),
            "op-x" => Some(&ram[_cOP_X.._cOP_X + _cOP_X_SIZE]),
            "op-y" => Some(&ram[_cOP_Y.._cOP_Y + _cOP_Y_SIZE]),
            "λ3e" => Some(&ram[_c_3E.._c_3E + _c_3E_SIZE]),
            "op-z" => Some(&ram[_cOP_Z.._cOP_Z + _cOP_Z_SIZE]),
            "op-bang" => Some(&ram[_cOP_BANG.._cOP_BANG + _cOP_BANG_SIZE]),
            "op-comment" => Some(&ram[_cOP_COMMENT.._cOP_COMMENT + _cOP_COMMENT_SIZE]),
            "end" => Some(&ram[_cEND.._cEND + _cEND_SIZE]),
            "op-midi" => Some(&ram[_cOP_MIDI.._cOP_MIDI + _cOP_MIDI_SIZE]),
            "has-bang" => Some(&ram[_cHAS_BANG.._cHAS_BANG + _cHAS_BANG_SIZE]),
            "has-pitch" => Some(&ram[_cHAS_PITCH.._cHAS_PITCH + _cHAS_PITCH_SIZE]),
            "chn" => Some(&ram[_cCHN.._cCHN + _cCHN_SIZE]),
            "pitch" => Some(&ram[_cPITCH.._cPITCH + _cPITCH_SIZE]),
            "len" => Some(&ram[_cLEN.._cLEN + _cLEN_SIZE]),
            "vel" => Some(&ram[_cVEL.._cVEL + _cVEL_SIZE]),
            "op-pitch" => Some(&ram[_cOP_PITCH.._cOP_PITCH + _cOP_PITCH_SIZE]),
            "has-note" => Some(&ram[_cHAS_NOTE.._cHAS_NOTE + _cHAS_NOTE_SIZE]),
            "is-bang" => Some(&ram[_cIS_BANG.._cIS_BANG + _cIS_BANG_SIZE]),
            "op-byte" => Some(&ram[_cOP_BYTE.._cOP_BYTE + _cOP_BYTE_SIZE]),
            "op-self" => Some(&ram[_cOP_SELF.._cOP_SELF + _cOP_SELF_SIZE]),
            "λ3f" => Some(&ram[_c_3F.._c_3F + _c_3F_SIZE]),
            "op-self/<push>" => {
                Some(&ram[_cOP_SELF__PUSH_.._cOP_SELF__PUSH_ + _cOP_SELF__PUSH__SIZE])
            }
            "op-self/ptr" => Some(&ram[_cOP_SELF_PTR.._cOP_SELF_PTR + _cOP_SELF_PTR_SIZE]),
            "op-null" => Some(&ram[_cOP_NULL.._cOP_NULL + _cOP_NULL_SIZE]),
            "help/lut" => Some(&ram[_cHELP_LUT.._cHELP_LUT + _cHELP_LUT_SIZE]),
            "midi/lut" => Some(&ram[_cMIDI_LUT.._cMIDI_LUT + _cMIDI_LUT_SIZE]),
            "chrb36/lut" => Some(&ram[_cCHRB36_LUT.._cCHRB36_LUT + _cCHRB36_LUT_SIZE]),
            "b36clc" => Some(&ram[_cB36CLC.._cB36CLC + _cB36CLC_SIZE]),
            "library/lut" => Some(&ram[_cLIBRARY_LUT.._cLIBRARY_LUT + _cLIBRARY_LUT_SIZE]),
            "select/icns" => Some(&ram[_cSELECT_ICNS.._cSELECT_ICNS + _cSELECT_ICNS_SIZE]),
            "beat-icn" => Some(&ram[_cBEAT_ICN.._cBEAT_ICN + _cBEAT_ICN_SIZE]),
            "font/glyphs" => Some(&ram[_cFONT_GLYPHS.._cFONT_GLYPHS + _cFONT_GLYPHS_SIZE]),
            "font/space" => Some(&ram[_cFONT_SPACE.._cFONT_SPACE + _cFONT_SPACE_SIZE]),
            "font/dot" => Some(&ram[_cFONT_DOT.._cFONT_DOT + _cFONT_DOT_SIZE]),
            "fill-icn" => Some(&ram[_cFILL_ICN.._cFILL_ICN + _cFILL_ICN_SIZE]),
            "appicon" => Some(&ram[_cAPPICON.._cAPPICON + _cAPPICON_SIZE]),
            "types/buf" => Some(&ram[_cTYPES_BUF.._cTYPES_BUF + _cTYPES_BUF_SIZE]),
            "grid/buf" => Some(&ram[_cGRID_BUF.._cGRID_BUF + _cGRID_BUF_SIZE]),
            "op-self/buf" => Some(&ram[_cOP_SELF_BUF.._cOP_SELF_BUF + _cOP_SELF_BUF_SIZE]),
            "edit/buf" => Some(&ram[_cEDIT_BUF.._cEDIT_BUF + _cEDIT_BUF_SIZE]),
            _ => None,
        }
    }
}
