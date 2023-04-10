#[macro_export]
macro_rules! bitloop {
    ($bb:expr, $sq:ident, $func:expr) => {
        while $bb > 0 {
            let $sq = $bb.trailing_zeros() as u16;
            $bb &= $bb - 1;
            $func;
        }
    };
}

#[macro_export]
macro_rules! init {
    ($i:ident, $($r:tt)+) => {{
        let mut $i = 0;
        let mut res = [{$($r)+}; 64];
        while $i < 64 {
            res[$i] = {$($r)+};
            $i += 1;
        }
        res
    }}
}
