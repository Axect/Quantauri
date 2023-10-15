import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import scienceplots
import argparse

args = argparse.ArgumentParser()
args.add_argument('--test', type=str, default='00')
args = args.parse_args()

test = args.test

period = 240
threshold = 5
if test == '00':
    period = 1
    threshold = 1
elif test == '01':
    period = 5
    threshold = 5
elif test == '02':
    period = 20
    threshold = 10
elif test == '03':
    period = 60
    threshold = 15
elif test == '04':
    period = 120
    threshold = 20
elif test == '05':
    period = 240
    threshold = 30

# Import parquet file
df_periodic = pd.read_parquet(f'./data/rebalance_test_periodic_{test}.parquet')
dg_periodic = pd.read_parquet(f'./data/rebalance_report_periodic_{test}.parquet')
df_threshold = pd.read_parquet(f'./data/rebalance_test_threshold_{test}.parquet')
dg_threshold = pd.read_parquet(f'./data/rebalance_report_threshold_{test}.parquet')

N = df_periodic.shape[0]

# Concatenate dataframes
df = pd.concat([df_periodic, df_threshold])
dg = pd.concat([dg_periodic, dg_threshold])

# Prepare Data to Plot
x = df_periodic['date'] # str
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

    p_str = f'P{period}'
    t_str = f'T{threshold}'

    values = dg.values
    for r in range(values.shape[0]):
        for i in range(1, values.shape[1]):
            values[r][i] = f'{values[r][i]:.4f}'
    axs[0].table(cellText=values, colLabels=dg.columns, loc='center', cellLoc='center')
    axs[0].axis('off')

    axs[1].plot(x, cumulative_return[:N], color='b', label=p_str)
    axs[1].plot(x, cumulative_return[N:], color='g', label=t_str)
    axs[1].axhline(y=1, color='k', linestyle='--')
    axs[1].set_ylabel('Cumulative Return')
    axs[1].set_ylim([0.5, 5])
    axs[1].grid(True)
    axs[1].legend()

    vol_mean_1 = np.mean(volatility[120:N])
    vol_mean_2 = np.mean(volatility[N+120:])
    axs[2].plot(x, volatility[:N], color='b')
    axs[2].plot(x, volatility[N:], color='g')
    axs[2].axhline(y=vol_mean_1, color='b', linestyle='--', label=f'{p_str} mean: {vol_mean_1:.2f}')
    axs[2].axhline(y=vol_mean_2, color='g', linestyle='--', label=f'{t_str} mean: {vol_mean_2:.2f}')
    axs[2].set_ylabel('Volatility (6 month)')
    axs[2].set_ylim([0, 1])
    axs[2].grid(True)
    axs[2].legend()

    sr_mean_1 = np.mean(sharpe_ratio[120:N])
    sr_mean_2 = np.mean(sharpe_ratio[N+120:])
    axs[3].plot(x, sharpe_ratio[:N], color='b')
    axs[3].plot(x, sharpe_ratio[N:], color='g')
    axs[3].axhline(y=sr_mean_1, color='b', linestyle='--', label=f'{p_str} mean: {sr_mean_1:.2f}')
    axs[3].axhline(y=sr_mean_2, color='g', linestyle='--', label=f'{t_str} mean: {sr_mean_2:.2f}')
    axs[3].axhline(y=0, color='k', linestyle='--')
    axs[3].set_ylabel('Sharpe Ratio (6 month)')
    axs[3].set_ylim([-5, 5])
    axs[3].grid(True)
    axs[3].legend()

    # Underwater plot
    axs[4].plot(x, -drawdown[:N] * 100, color='b')
    axs[4].plot(x, -drawdown[N:] * 100, color='g')
    axs[4].axhline(y=0, color='k', linestyle='--')
    axs[4].fill_between(x, -drawdown[:N] * 100, 0, color='b', alpha=0.3)
    axs[4].fill_between(x, -drawdown[N:] * 100, 0, color='g', alpha=0.3)
    axs[4].axhline(y=-np.max(drawdown[:N]) * 100, color='b', linestyle='--', label=f'P60 MDD: {-np.max(drawdown[:N]) * 100:.2f}\%')
    axs[4].axhline(y=-np.max(drawdown[N:]) * 100, color='g', linestyle='--', label=f'T10 MDD: {-np.max(drawdown[N:]) * 100:.2f}\%')
    axs[4].set_ylabel('Drawdown (\%)')
    axs[4].set_ylim([-100, 5])
    axs[4].grid(True)
    axs[4].legend()

    # Balance History
    axs[5].plot(x, balance_history[:N], '.-', color='b', label=p_str, alpha=0.5)
    axs[5].plot(x, balance_history[N:], '.-', color='g', label=t_str, alpha=0.5)
    axs[5].set_xlabel('Date')
    axs[5].set_ylabel('Balance')
    axs[5].grid(True)

    fig.tight_layout()
    # Remove margin between table and plot
    #fig.subplots_adjust(hspace=0)
    fig.align_ylabels()
    plt.savefig(f'figs/rebalance_test_{test}.png', dpi=300)

