import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import scienceplots
import argparse

args = argparse.ArgumentParser()
args.add_argument('--ticker', type=str, default='005930.KS')
args = args.parse_args()

symbol = args.ticker

# Import parquet file
df = pd.read_parquet(f'data/{symbol}_strategy.parquet')
df_summary = pd.read_parquet(f'data/{symbol}_summary.parquet')

# Prepare Data to Plot
x = df['date'] # str
x = pd.to_datetime(x) # datetime

bnh_cr = df['bnh_cr']
bnh_vol = df['bnh_vol']
bnh_sr = df['bnh_sr']
bnh_dd = df['bnh_dd']

ma_co_cr = df['ma_co_cr']
ma_co_vol = df['ma_co_vol']
ma_co_sr = df['ma_co_sr']
ma_co_dd = df['ma_co_dd']

macd_adx_cr = df['macd_adx_cr']
macd_adx_vol = df['macd_adx_vol']
macd_adx_sr = df['macd_adx_sr']
macd_adx_dd = df['macd_adx_dd']

# Plot
with plt.style.context(["science", "nature"]):
    fig, axs = plt.subplots(5, 1, figsize=(6, 8), sharex=True, gridspec_kw={'height_ratios': [1,2,2,2,2]})
    fig.suptitle(f'Various Strategies for {symbol}')

    values = df_summary.values
    for r in range(values.shape[0]):
        for i in range(1, values.shape[1]):
            values[r][i] = f'{values[r][i]:.4f}'
    axs[0].table(cellText=values, colLabels=df_summary.columns, loc='center', cellLoc='center')
    axs[0].axis('off')

    axs[1].plot(x, bnh_cr, color='b', label='Buy and Hold')
    axs[1].plot(x, ma_co_cr, color='r', label='MA Crossover')
    axs[1].plot(x, macd_adx_cr, color='g', label='MACD + ADX')
    axs[1].axhline(y=1, color='k', linestyle='--')
    axs[1].set_ylabel('Cumulative Return')
    axs[1].grid(True)
    axs[1].legend()

    vol_mean = np.mean(bnh_vol[120:])
    ma_co_vol_mean = np.mean(ma_co_vol[120:])
    macd_adx_vol_mean = np.mean(macd_adx_vol[120:])
    axs[2].plot(x, bnh_vol, color='b', label='Buy and Hold')
    axs[2].plot(x, ma_co_vol, color='r', label='MA Crossover')
    axs[2].plot(x, macd_adx_vol, color='g', label='MACD + ADX')
    axs[2].axhline(y=vol_mean, color='b', linestyle='--', label=f'BnH: {vol_mean:.2f}')
    axs[2].axhline(y=ma_co_vol_mean, color='r', linestyle='--', label=f'MA: {ma_co_vol_mean:.2f}')
    axs[2].axhline(y=macd_adx_vol_mean, color='g', linestyle='--', label=f'MX: {macd_adx_vol_mean:.2f}')
    axs[2].set_ylabel('Volatility (6 month)')
    axs[2].grid(True)
    axs[2].legend()

    sr_mean = np.mean(bnh_sr[120:])
    ma_co_sr_mean = np.mean(ma_co_sr[120:])
    macd_adx_sr_mean = np.mean(macd_adx_sr[120:])
    axs[3].plot(x, bnh_sr, color='b', label='Buy and Hold')
    axs[3].plot(x, ma_co_sr, color='r', label='MA Crossover')
    axs[3].plot(x, macd_adx_sr, color='g', label='MACD + ADX')
    axs[3].axhline(y=sr_mean, color='b', linestyle='--', label=f'BnH: {sr_mean:.2f}')
    axs[3].axhline(y=ma_co_sr_mean, color='r', linestyle='--', label=f'MA: {ma_co_sr_mean:.2f}')
    axs[3].axhline(y=macd_adx_sr_mean, color='g', linestyle='--', label=f'MX: {macd_adx_sr_mean:.2f}')
    axs[3].axhline(y=0, color='k', linestyle='--')
    axs[3].set_ylabel('Sharpe Ratio (6 month)')
    axs[3].grid(True)
    axs[3].legend()

    # Underwater plot
    axs[4].plot(x, -bnh_dd * 100, color='b')
    axs[4].plot(x, -ma_co_dd * 100, color='r')
    axs[4].plot(x, -macd_adx_dd * 100, color='g')
    axs[4].axhline(y=0, color='k', linestyle='--')
    axs[4].fill_between(x, -bnh_dd * 100, 0, color='b', alpha=0.3)
    axs[4].fill_between(x, -ma_co_dd * 100, 0, color='r', alpha=0.3)
    axs[4].fill_between(x, -macd_adx_dd * 100, 0, color='g', alpha=0.3)
    axs[4].axhline(y=-np.max(bnh_dd) * 100, color='b', linestyle='--', label=f'BnH: {-np.max(bnh_dd) * 100:.2f}\%')
    axs[4].axhline(y=-np.max(ma_co_dd) * 100, color='r', linestyle='--', label=f'MA: {-np.max(ma_co_dd) * 100:.2f}\%')
    axs[4].axhline(y=-np.max(macd_adx_dd) * 100, color='g', linestyle='--', label=f'MX: {-np.max(macd_adx_dd) * 100:.2f}\%')
    axs[4].set_xlabel('Date')
    axs[4].set_ylabel('Drawdown (\%)')
    axs[4].grid(True)
    axs[4].legend()

    fig.tight_layout()
    # Remove margin between table and plot
    #fig.subplots_adjust(hspace=0)
    fig.align_ylabels()
    plt.savefig(f'figs/{symbol}_strategy.png', dpi=300)

