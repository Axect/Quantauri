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
df = pd.read_parquet(f'data/{symbol}.parquet')

# Prepare Data to Plot
x = df['date'] # str
x = pd.to_datetime(x) # datetime
tp = df['tp']
tp_div = df['tp_div']
tp_slope = df['tp_slope']
sma = df['sma']
ema = df['ema']
wma = df['wma']
rsi = df['rsi']
rsi_signal = df['rsi_signal']
rsi_div = df['rsi_div']
rsi_slope = df['rsi_slope']
macd = df['macd']
macd_signal = df['macd_signal']
adx = df['adx']
di_plus = df['di_plus']
di_minus = df['di_minus']
k = df['k']
d = df['d']
cci = df['cci']

# Plot
with plt.style.context(["science", "nature"]):
    fig, axs = plt.subplots(7, 1, figsize=(6, 8), sharex=True, gridspec_kw={'height_ratios': [4, 1, 1, 1, 1, 1, 1]})
    axs[0].autoscale(tight=True)
    axs[0].plot(x, tp, label='Typical Price')
    axs[0].plot(x, sma, '--', label='SMA(20)')
    axs[0].plot(x, ema, '-.', label='EMA(20)')
    axs[0].plot(x, wma, ':', label='WMA(20)')
    axs[0].plot(x, tp_div, '--', label='TP SLM')
    # Tick angle
    plt.setp(axs[0].get_xticklabels(), rotation=30, horizontalalignment='right')
    axs[0].legend()
    axs[0].grid(True)
    axs[0].set_ylabel("Price")

    axs[1].autoscale(tight=True)
    axs[1].plot(x, rsi)
    axs[1].plot(x, rsi_signal, '-.')
    axs[1].plot(x, rsi_div, '--')
    axs[1].axhline(y=30, color='r', linestyle='--')
    axs[1].axhline(y=70, color='r', linestyle='--')
    axs[1].set_ylabel("RSI")
    axs[1].set_ylim([10, 90])
    axs[1].grid(True)

    axs[2].autoscale(tight=True)
    rsi_slope = np.tanh(np.array(rsi_slope) * np.array(tp_slope) / np.max(tp_slope) * 2)
    rsi_slope_up = np.array(rsi_slope)
    rsi_slope_up[rsi_slope_up < 0] = np.nan
    rsi_slope_down = np.array(rsi_slope)
    rsi_slope_down[rsi_slope_down > 0] = np.nan
    axs[2].bar(x, rsi_slope_up, color='r', width=0.8)
    axs[2].bar(x, rsi_slope_down, color='b', width=0.8)
    axs[2].set_ylim([-1, 1])
    axs[2].set_ylabel("RSI Div")
    axs[2].grid(True)

    axs[3].autoscale(tight=True)
    #axs[3].plot(x, macd)
    #axs[3].plot(x, macd_signal, '-.')
    axs[3].axhline(y=0, color='k', linestyle='--')
    # Histogram for MACD
    macd_up = np.tanh(np.array(macd - macd_signal) / np.max(macd_signal) * 7.5)
    macd_up[macd_up < 0] = np.nan
    macd_down = np.tanh(np.array(macd - macd_signal) / np.max(macd_signal) * 7.5)
    macd_down[macd_down > 0] = np.nan
    axs[3].set_ylim([-1, 1])
    axs[3].bar(x, macd_up, color='r', width=0.8)
    axs[3].bar(x, macd_down, color='b', width=0.8)
    axs[3].axhline(y=0.5, color='r', linestyle='--')
    axs[3].axhline(y=-0.5, color='b', linestyle='--')
    axs[3].set_ylabel("MACD Osc")
    axs[3].grid(True)

    axs[4].autoscale(tight=True)
    axs[4].plot(x, adx, 'k')
    axs[4].axhline(y=20, color='b', linestyle='--')
    axs[4].axhline(y=25, color='r', linestyle='--')
    axs[4].set_ylabel("ADX")
    axs[4].set_ylim([0, 80])
    axs[4].grid(True)

    axs[5].autoscale(tight=True)
    di_up = np.tanh(np.array(di_plus - di_minus) / 50)
    di_up[di_up < 0] = np.nan
    di_down = np.tanh(np.array(di_plus - di_minus) / 50)
    di_down[di_down > 0] = np.nan
    axs[5].bar(x, di_up, color='r', width=0.8)
    axs[5].bar(x, di_down, color='b', width=0.8)
    axs[5].set_ylim([-1, 1])
    axs[5].set_ylabel("DI+/-")
    axs[5].grid(True)

    axs[6].autoscale(tight=True)
    axs[6].plot(x, cci / 100, label='CCI')
    axs[6].axhline(y=1, color='r', linestyle='--')
    axs[6].axhline(y=0, color='k', linestyle='--')
    axs[6].axhline(y=-1, color='b', linestyle='--')
    axs[6].set_xlabel("Date")
    axs[6].set_ylabel("CCI/100")
    axs[6].grid(True)

    #axs[6].autoscale(tight=True)
    #axs[6].plot(x, k, label='\%K')
    #axs[6].plot(x, d, label='\%D')
    #axs[6].axhline(y=20, color='b', linestyle='--')
    #axs[6].axhline(y=80, color='r', linestyle='--')
    #axs[6].legend()
    #axs[6].set_xlabel("Date")
    #axs[6].set_ylabel("Stochastic")
    #axs[6].set_ylim([0, 100])
    #axs[6].grid(True)

    fig.tight_layout()
    fig.savefig(f'figs/{symbol}.png', dpi=300, bbox_inches='tight')
