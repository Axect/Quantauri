import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import scienceplots
import argparse

args = argparse.ArgumentParser()
args.add_argument('--test', type=str, default='00')
args = args.parse_args()

test = args.test

# Import parquet file
df = pd.read_parquet(f'./data/trade_test_{test}.parquet')
dg = pd.read_parquet(f'./data/trade_report_{test}.parquet')

# Prepare Data to Plot
x = df['date'] # str
x = pd.to_datetime(x) # datetime

daily_return = df['daily_return']
cumulative_return = df['cumulative_return']
volatility = df['rolling_volatility']
sharpe_ratio = df['rolling_sharpe_ratio']
drawdown = df['drawdown']
balance_history = df['balance_history']

# Plot
with plt.style.context(["science", "nature"]):
    fig, axs = plt.subplots(6, 1, figsize=(6, 8), sharex=True, gridspec_kw={'height_ratios': [1,2,2,2,2,2]})
    #fig.suptitle(f'Various Strategies for {symbol}')

    values = dg.values
    #for r in range(values.shape[0]):
    for i in range(1, values.shape[1]):
        values[0][i] = f'{values[0][i]:.4f}'
    axs[0].table(cellText=values, colLabels=dg.columns, loc='center', cellLoc='center')
    axs[0].axis('off')

    axs[1].plot(x, cumulative_return, color='b', label='BnH (Rebalancing = 3 month)')
    axs[1].axhline(y=1, color='k', linestyle='--')
    axs[1].set_ylabel('Cumulative Return')
    axs[1].set_ylim([0.5, 2])
    axs[1].grid(True)
    axs[1].legend()

    vol_mean = np.mean(volatility[120:])
    axs[2].plot(x, volatility, color='b', label='BnH (Rebalancing = 3 month)')
    axs[2].axhline(y=vol_mean, color='r', linestyle='--', label=f'Mean: {vol_mean:.2f}')
    axs[2].set_ylabel('Volatility (6 month)')
    axs[2].set_ylim([0, 1])
    axs[2].grid(True)
    axs[2].legend()

    sr_mean = np.mean(sharpe_ratio[120:])
    axs[3].plot(x, sharpe_ratio, color='b', label='BH (Rebalancing = 3 month)')
    axs[3].axhline(y=sr_mean, color='r', linestyle='--', label=f'Mean: {sr_mean:.2f}')
    axs[3].axhline(y=0, color='k', linestyle='--')
    axs[3].set_ylabel('Sharpe Ratio (6 month)')
    axs[3].set_ylim([-5, 5])
    axs[3].grid(True)
    axs[3].legend()

    # Underwater plot
    axs[4].plot(x, -drawdown * 100, color='r')
    axs[4].axhline(y=0, color='k', linestyle='--')
    axs[4].fill_between(x, -drawdown * 100, 0, color='r', alpha=0.3)
    axs[4].axhline(y=-np.max(drawdown) * 100, color='b', linestyle='--', label=f'MDD: {-np.max(drawdown) * 100:.2f}\%')
    axs[4].set_ylabel('Drawdown (\%)')
    axs[4].set_ylim([-100, 5])
    axs[4].grid(True)
    axs[4].legend()

    # Balance History
    axs[5].plot(x, balance_history, '.-', color='b')
    axs[5].set_xlabel('Date')
    axs[5].set_ylabel('Balance')
    axs[5].grid(True)

    fig.tight_layout()
    # Remove margin between table and plot
    #fig.subplots_adjust(hspace=0)
    fig.align_ylabels()
    plt.savefig(f'figs/trade_test_{test}.png', dpi=300)

