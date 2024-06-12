use peroxide::fuga::*;

/// Simple Moving Average
pub fn sma(v: &[f64], period: usize) -> Vec<f64> {
    let mut res = vec![0f64; v.len()];
    let mut sum = 0.0;
    for i in 0..v.len() {
        sum += v[i];
        if i >= period {
            sum -= v[i - period];
            res[i] = sum / period as f64;
        } else {
            res[i] = sum / (i + 1) as f64;
        }
    }
    res
}

/// Exponential Moving Average
pub fn ema(v: &[f64], period: usize) -> Vec<f64> {
    let mut res = vec![0f64; v.len()];
    res[0] = v[0];
    let mut ema = v[0];
    let alpha = 2.0 / (period as f64 + 1.0);
    for i in 1..v.len() {
        ema = v[i] * alpha + ema * (1.0 - alpha);
        res[i] = ema;
    }
    res
}

/// Weighted Moving Average
pub fn wma(v: &[f64], period: usize) -> Vec<f64> {
    let mut res = vec![0f64; v.len()];
    let denom = ((period * (period + 1)) / 2) as f64;
    let mut total = 0f64;
    let mut numer = 0f64;
    for i in 0..v.len() {
        if i >= period {
            numer = numer + period as f64 * v[i] - total;
            total = total + v[i] - v[i - period];
            res[i] = numer / denom;
        } else {
            numer = numer + (i + 1) as f64 * v[i];
            total = total + v[i];
            res[i] = numer / (((i + 1) * (i + 2)) / 2) as f64;
        }
    }
    res
}

/// Double Exponential Moving Average
pub fn dema(v: &[f64], period: usize) -> Vec<f64> {
    let ema_ = ema(v, period);
    let ema_ema = ema(&ema_, period);
    zip_with(|x, y| 2.0 * x - y, &ema_, &ema_ema)
}

/// Triple Exponential Moving Average
pub fn tema(v: &[f64], period: usize) -> Vec<f64> {
    let ema_ = ema(v, period);
    let ema_ema = ema(&ema_, period);
    let ema_ema_ema = ema(&ema_ema, period);
    ema_.iter()
        .zip(ema_ema.iter())
        .zip(ema_ema_ema.iter())
        .map(|((x, y), z)| 3.0 * x - 3.0 * y + z)
        .collect::<Vec<f64>>()
}

/// Williams %R
pub fn williams_r(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let mut res = vec![0f64; high.len()];
    for i in 0..high.len() {
        let mut highest = high[i];
        let mut lowest = low[i];
        for j in 1..period {
            if i >= j {
                highest = highest.max(high[i - j]);
                lowest = lowest.min(low[i - j]);
            }
        }
        res[i] = (highest - close[i]) / (highest - lowest) * -100.0;
    }
    res
}

/// Smoothed Moving Average
pub fn smma(v: &[f64], period: usize) -> Vec<f64> {
    ema(v, 2 * period - 1)
}

/// Relative Strength Index
pub fn rsi(v: &[f64], period: usize) -> Vec<f64> {
    let mut u = vec![0f64; v.len()];
    let mut d = vec![0f64; v.len()];
    for i in 1..v.len() {
        let diff = v[i] - v[i - 1];
        if diff > 0.0 {
            u[i] = diff;
        } else {
            d[i] = -diff;
        }
    }
    let au = smma(&u, period);
    let ad = smma(&d, period);
    zip_with(|x, y| 100f64 * x / (x + y + 1e-3), &au, &ad)
}

/// Moving Average Convergence Divergence
pub fn macd(v: &[f64], period1: usize, period2: usize) -> Vec<f64> {
    let ema1 = ema(v, period1);
    let ema2 = ema(v, period2);
    zip_with(|x, y| x - y, &ema1, &ema2)
}

/// Average True Range
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let mut tr = vec![0f64; high.len()];
    for i in 1..high.len() {
        let h_pc = high[i].max(close[i - 1]);
        let l_pc = low[i].min(close[i - 1]);
        tr[i] = h_pc - l_pc;
    }
    let mut atr = vec![0f64; high.len()];
    let mut sum = 0f64;
    for i in 0..period {
        sum += tr[i];
        atr[i] = sum / (i + 1) as f64;
    }
    for i in period..high.len() {
        atr[i] = (atr[i - 1] * (period - 1) as f64 + tr[i]) / period as f64;
    }
    atr
}

/// Average Directional movement Index & Directional Movement Index
#[allow(unused_assignments)]
pub fn adx_dmi(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut up_move = 0f64;
    let mut down_move = 0f64;
    let mut dm_plus = vec![0f64; high.len()];
    let mut dm_minus = vec![0f64; high.len()];
    for i in 1..high.len() {
        up_move = high[i] - high[i - 1];
        down_move = low[i - 1] - low[i];

        if up_move > down_move && up_move > 0f64 {
            dm_plus[i] = up_move;
        } else {
            dm_plus[i] = 0f64;
        }

        if down_move > up_move && down_move > 0f64 {
            dm_minus[i] = down_move;
        } else {
            dm_minus[i] = 0f64;
        }
    }

    if dm_plus[1] > 0f64 {
        dm_plus[0] = dm_plus[1];
    } else if dm_minus[1] > 0f64 {
        dm_minus[0] = dm_minus[1];
    } else {
        dm_plus[0] = 0.1f64;
        dm_minus[0] = 0.1f64;
    }

    let dm_plus_ema = ema(&dm_plus, period);
    let dm_minus_ema = ema(&dm_minus, period);
    let mut atr = atr(high, low, close, period);

    if atr[0] == 0f64 {
        atr[0] = atr[1];
    }

    let (di_plus, di_minus): (Vec<f64>, Vec<f64>) = atr
        .iter()
        .zip(dm_plus_ema.iter().zip(dm_minus_ema.iter()))
        .map(|(x, (y, z))| (100f64 * y / x, 100f64 * z / x))
        .unzip();

    let dx = zip_with(|x, y| (x - y).abs() / (x + y), &di_plus, &di_minus);
    (ema(&dx, period).fmap(|x| 100f64 * x), di_plus, di_minus)
}

/// Stochastic Oscillator
pub fn stochastic(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
    smooth: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut k = vec![0f64; high.len()];
    for i in 0..high.len() {
        let mut highest = high[i];
        let mut lowest = low[i];
        for j in 1..period {
            if i >= j {
                highest = highest.max(high[i - j]);
                lowest = lowest.min(low[i - j]);
            }
        }
        k[i] = (close[i] - lowest) / (highest - lowest) * 100f64;
    }
    let d = sma(&k, smooth);
    (k, d)
}

/// Divergence compute for any indicator
/// Return: (Value, Slope)
pub fn divergence(v: &[f64]) -> (Vec<f64>, Vec<f64>) {
    // Find all local maxima
    let mut maxima = vec![];
    let mut max_idx = 0usize;
    let mut max_val = v[0];
    for i in 1..v.len() {
        if v[i] > max_val {
            max_val = v[i];
            max_idx = i;
        } else if v[i] < max_val {
            if max_idx > 0 {
                maxima.push((max_idx, max_val));
            }
            max_idx = 0;
            max_val = v[i];
        }
    }

    let (idx_f64, maxima): (Vec<f64>, Vec<f64>) =
        maxima.into_iter().map(|(x, y)| (x as f64, y)).unzip();

    // Find all local maxima of local maxima
    let mut maxima2 = vec![];
    let mut max_idx = 0usize;
    let mut max_val = maxima[0];
    for i in 1..maxima.len() {
        if maxima[i] > max_val {
            max_val = maxima[i];
            max_idx = i;
        } else if maxima[i] < max_val {
            if max_idx > 0 {
                maxima2.push((max_idx, max_val));
            }
            max_idx = 0;
            max_val = maxima[i];
        }
    }

    let (mut idx_f64, mut maxima): (Vec<f64>, Vec<f64>) =
        maxima2.into_iter().map(|(x, y)| (idx_f64[x], y)).unzip();

    // Insert first & last point if it is not included
    if idx_f64[0] != 0f64 {
        idx_f64.insert(0, 0f64);
        maxima.insert(0, v[0]);
    }
    if idx_f64[idx_f64.len() - 1] != v.len() as f64 - 1f64 {
        idx_f64.push(v.len() as f64 - 1f64);
        maxima.push(v[v.len() - 1]);
    }

    // Create cubic spline of local maxima
    let cs = cubic_hermite_spline(&idx_f64, &maxima, Akima).unwrap();
    let idx = seq(0, v.len() as u32 - 1, 1);
    let div = cs.eval_vec(&idx.fmap(|x| x as f64));
    let slope = cs.derivative().eval_vec(&idx.fmap(|x| x as f64));
    (div, slope)
}

/// Commodity Channel Index
pub fn cci(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let mut tp = vec![0f64; high.len()];
    for i in 0..high.len() {
        tp[i] = (high[i] + low[i] + close[i]) / 3f64;
    }
    let m = sma(&tp, period);
    let deviation = zip_with(|x, y| x - y, &tp, &m);
    let d = sma(&deviation.fmap(|x| x.abs()), period);
    zip_with(|x, y| (x - y) / (0.015 * y), &deviation, &d)
}
