use super::tuner_eval::ParamContainer;

impl ParamContainer {
    pub const fn old() -> Self {
        ParamContainer {
            doubled_mg: 0,
            doubled_eg: -5,
            isolated_mg: -20,
            isolated_eg: -5,
            passed_mg: -5,
            passed_eg: 70,
            shield_mg: 5,
            shield_eg: 2,
            open_file_mg: -10,
            open_file_eg: 5,
        }
    }

    pub const fn new() -> Self {
        ParamContainer {
            doubled_mg: 1,
            doubled_eg: -15,
            isolated_mg: -8,
            isolated_eg: -5,
            passed_mg: -7,
            passed_eg: 27,
            shield_mg: 2,
            shield_eg: 1,
            open_file_mg: -10,
            open_file_eg: 8,
        }
    }
}