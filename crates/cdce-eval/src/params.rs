use gears::games::chess::pieces::PieceType;
use gears::games::chess::pieces::PieceType::*;
use gears::general::common::anyhow::anyhow;
use gears::general::common::{parse_int_from_str, Res};
use gears::ugi::{EngineOption, EngineOptionName, EngineOptionNameForProtocol, EngineOptionType, Protocol, UgiSpin};

/// All tunable weights and parameters for the CDCE evaluation function.
#[derive(Debug, Clone)]
pub struct CdceParams {
    // Material piece values
    pub val_pawn: i32,
    pub val_knight: i32,
    pub val_bishop: i32,
    pub val_rook: i32,
    pub val_queen: i32,

    // One-way attack weights (%)
    pub ow_weight_knight: i32,
    pub ow_weight_bishop: i32,
    pub ow_weight_rook: i32,
    pub ow_weight_queen: i32,
    pub ow_weight_pawn: i32,
    pub ow_weight_king: i32,

    // Precomputed 6x6 table: OW[attacker][target]
    pub ow_table: [[i32; 6]; 6],

    // Knight forks
    pub fork_bonus: i32,

    // Rook activity
    pub rook_off_home_rank: i32,
    pub rook_rank3: i32,
    pub rook_file_open: i32,
    pub rook_file_semi_open: i32,
    pub rook_connected: i32,
    pub rook_pawn_gate: i32,
    pub rook_pawn_phase_gate: i32,

    // Standard terms: Mobility
    pub mobility_knight: i32,
    pub mobility_bishop: i32,
    pub mobility_rook: i32,
    pub mobility_queen: i32,

    // Standard terms: King safety
    pub king_attacker_knight: i32,
    pub king_attacker_bishop: i32,
    pub king_attacker_rook: i32,
    pub king_attacker_queen: i32,
    pub king_shield: i32,

    // Standard terms: Pawn structure
    pub pawn_doubled: i32,
    pub pawn_isolated: i32,
    pub pawn_passed_rank: [i32; 8],

    // Tempo
    pub tempo: i32,
}

impl Default for CdceParams {
    fn default() -> Self {
        let mut params = Self {
            val_pawn: 100,
            val_knight: 320,
            val_bishop: 330,
            val_rook: 500,
            val_queen: 900,

            ow_weight_knight: 12,
            ow_weight_bishop: 6,
            ow_weight_rook: 6,
            ow_weight_queen: 3,
            ow_weight_pawn: 4,
            ow_weight_king: 0,

            ow_table: [[0; 6]; 6],

            fork_bonus: 40,

            rook_off_home_rank: 25,
            rook_rank3: 20,
            rook_file_open: 25,
            rook_file_semi_open: 12,
            rook_connected: 10,
            rook_pawn_gate: 8,
            rook_pawn_phase_gate: 14,

            mobility_knight: 4,
            mobility_bishop: 5,
            mobility_rook: 3,
            mobility_queen: 1,

            king_attacker_knight: 2,
            king_attacker_bishop: 2,
            king_attacker_rook: 3,
            king_attacker_queen: 5,
            king_shield: 10,

            pawn_doubled: 15,
            pawn_isolated: 20,
            pawn_passed_rank: [0, 0, 0, 10, 20, 40, 80, 150],

            tempo: 10,
        };
        params.recompute_ow_table();
        params
    }
}

impl CdceParams {
    /// Named value set `standard`: P100 N320 B330 R500 Q900 (classical values).
    /// Kept for callers like `cdce-tune`; matches `default()` values.
    pub fn standard() -> Self {
        let mut params = Self::default();
        params.set_option("Val_Knight", "320").expect("valid value");
        params.set_option("Val_Rook", "500").expect("valid value");
        params.set_option("Val_Queen", "900").expect("valid value");
        params
    }
}

impl CdceParams {
    /// Piece value for a given piece type (King is 0).
    #[inline]
    pub fn piece_value(&self, piece: PieceType) -> i32 {
        match piece {
            Pawn => self.val_pawn,
            Knight => self.val_knight,
            Bishop => self.val_bishop,
            Rook => self.val_rook,
            Queen => self.val_queen,
            King | Empty => 0,
        }
    }

    /// One-way attacker weight for a given piece type.
    #[inline]
    pub fn one_way_weight(&self, piece: PieceType) -> i32 {
        match piece {
            Pawn => self.ow_weight_pawn,
            Knight => self.ow_weight_knight,
            Bishop => self.ow_weight_bishop,
            Rook => self.ow_weight_rook,
            Queen => self.ow_weight_queen,
            King | Empty => 0,
        }
    }

    /// Rebuild the 6x6 OW[attacker][target] table:
    /// OW[a][t] = ONE_WAY_WEIGHT[a] * VALUE[t] / 100 (integer division).
    pub fn recompute_ow_table(&mut self) {
        let pieces = [Pawn, Knight, Bishop, Rook, Queen, King];
        for a in pieces {
            let w = self.one_way_weight(a);
            for t in pieces {
                let v = self.piece_value(t);
                self.ow_table[a as usize][t as usize] = (w * v) / 100;
            }
        }
    }

    /// Generate UCI spin options for engine options listing.
    pub fn ugi_options(&self) -> Vec<EngineOption> {
        let make_spin = |name: &'static str, val: i32, min: i32, max: i32| EngineOption {
            name: EngineOptionNameForProtocol {
                name: EngineOptionName::Other(name.to_string()),
                proto: Protocol::UGI,
            },
            value: EngineOptionType::Spin(UgiSpin {
                val: val as i64,
                default: Some(val as i64),
                min: Some(min as i64),
                max: Some(max as i64),
            }),
        };

        vec![
            make_spin("Val_Pawn", self.val_pawn, 0, 10000),
            make_spin("Val_Knight", self.val_knight, 0, 10000),
            make_spin("Val_Bishop", self.val_bishop, 0, 10000),
            make_spin("Val_Rook", self.val_rook, 0, 10000),
            make_spin("Val_Queen", self.val_queen, 0, 10000),
            make_spin("OW_Weight_Knight", self.ow_weight_knight, 0, 100),
            make_spin("OW_Weight_Bishop", self.ow_weight_bishop, 0, 100),
            make_spin("OW_Weight_Rook", self.ow_weight_rook, 0, 100),
            make_spin("OW_Weight_Queen", self.ow_weight_queen, 0, 100),
            make_spin("OW_Weight_Pawn", self.ow_weight_pawn, 0, 100),
            make_spin("Fork_Bonus", self.fork_bonus, 0, 1000),
            make_spin("Rook_OffHomeRank", self.rook_off_home_rank, 0, 1000),
            make_spin("Rook_Rank3", self.rook_rank3, 0, 1000),
            make_spin("Rook_FileOpen", self.rook_file_open, 0, 1000),
            make_spin("Rook_FileSemiOpen", self.rook_file_semi_open, 0, 1000),
            make_spin("Rook_Connected", self.rook_connected, 0, 1000),
            make_spin("Rook_PawnGate", self.rook_pawn_gate, 0, 1000),
            make_spin("Rook_PawnPhaseGate", self.rook_pawn_phase_gate, 0, 24),
            make_spin("Mobility_Knight", self.mobility_knight, 0, 100),
            make_spin("Mobility_Bishop", self.mobility_bishop, 0, 100),
            make_spin("Mobility_Rook", self.mobility_rook, 0, 100),
            make_spin("Mobility_Queen", self.mobility_queen, 0, 100),
            make_spin("King_Attacker_Knight", self.king_attacker_knight, 0, 100),
            make_spin("King_Attacker_Bishop", self.king_attacker_bishop, 0, 100),
            make_spin("King_Attacker_Rook", self.king_attacker_rook, 0, 100),
            make_spin("King_Attacker_Queen", self.king_attacker_queen, 0, 100),
            make_spin("King_Shield", self.king_shield, 0, 100),
            make_spin("Pawn_Doubled", self.pawn_doubled, 0, 1000),
            make_spin("Pawn_Isolated", self.pawn_isolated, 0, 1000),
            make_spin("Pawn_Passed_Rank4", self.pawn_passed_rank[4], 0, 1000),
            make_spin("Pawn_Passed_Rank5", self.pawn_passed_rank[5], 0, 1000),
            make_spin("Pawn_Passed_Rank6", self.pawn_passed_rank[6], 0, 1000),
            make_spin("Pawn_Passed_Rank7", self.pawn_passed_rank[7], 0, 1000),
            make_spin("Tempo", self.tempo, 0, 1000),
        ]
    }

    /// Update a parameter by name and rebuild dependent tables.
    pub fn set_option(&mut self, name: &str, value: &str) -> Res<()> {
        let val: i32 = parse_int_from_str(value, name)?;
        let mut recompute_ow = false;

        match name.to_ascii_lowercase().as_str() {
            "val_pawn" => {
                self.val_pawn = val;
                recompute_ow = true;
            }
            "val_knight" => {
                self.val_knight = val;
                recompute_ow = true;
            }
            "val_bishop" => {
                self.val_bishop = val;
                recompute_ow = true;
            }
            "val_rook" => {
                self.val_rook = val;
                recompute_ow = true;
            }
            "val_queen" => {
                self.val_queen = val;
                recompute_ow = true;
            }
            "ow_weight_knight" | "ow_knight" => {
                self.ow_weight_knight = val;
                recompute_ow = true;
            }
            "ow_weight_bishop" | "ow_bishop" => {
                self.ow_weight_bishop = val;
                recompute_ow = true;
            }
            "ow_weight_rook" | "ow_rook" => {
                self.ow_weight_rook = val;
                recompute_ow = true;
            }
            "ow_weight_queen" | "ow_queen" => {
                self.ow_weight_queen = val;
                recompute_ow = true;
            }
            "ow_weight_pawn" | "ow_pawn" => {
                self.ow_weight_pawn = val;
                recompute_ow = true;
            }
            "ow_weight_king" | "ow_king" => {
                self.ow_weight_king = val;
                recompute_ow = true;
            }
            "fork_bonus" => self.fork_bonus = val,
            "rook_offhomerank" | "rook_off_home_rank" => self.rook_off_home_rank = val,
            "rook_rank3" | "rook_relative_rank3" => self.rook_rank3 = val,
            "rook_fileopen" | "rook_file_open" => self.rook_file_open = val,
            "rook_filesemiopen" | "rook_file_semi_open" => self.rook_file_semi_open = val,
            "rook_connected" => self.rook_connected = val,
            "rook_pawngate" | "rook_pawn_gate" => self.rook_pawn_gate = val,
            "rook_pawnphasegate" | "rook_pawn_phase_gate" => self.rook_pawn_phase_gate = val,
            "mobility_knight" => self.mobility_knight = val,
            "mobility_bishop" => self.mobility_bishop = val,
            "mobility_rook" => self.mobility_rook = val,
            "mobility_queen" => self.mobility_queen = val,
            "king_attacker_knight" => self.king_attacker_knight = val,
            "king_attacker_bishop" => self.king_attacker_bishop = val,
            "king_attacker_rook" => self.king_attacker_rook = val,
            "king_attacker_queen" => self.king_attacker_queen = val,
            "king_shield" => self.king_shield = val,
            "pawn_doubled" => self.pawn_doubled = val,
            "pawn_isolated" => self.pawn_isolated = val,
            "pawn_passed_rank4" => self.pawn_passed_rank[4] = val,
            "pawn_passed_rank5" => self.pawn_passed_rank[5] = val,
            "pawn_passed_rank6" => self.pawn_passed_rank[6] = val,
            "pawn_passed_rank7" => self.pawn_passed_rank[7] = val,
            "tempo" => self.tempo = val,
            _ => return Err(anyhow!("Unknown CDCE option: {name}")),
        }
        if recompute_ow {
            self.recompute_ow_table();
        }
        Ok(())
    }
}
