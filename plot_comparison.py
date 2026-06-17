import json
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np

with open('assets/comparison_v86_v89.json') as f:
    data = json.load(f)

fig, axes = plt.subplots(2, 4, figsize=(20, 10))
axes = axes.flatten()

colors = {'v86': '#4CAF50', 'v89': '#2196F3', 'lkh': '#FF9800'}
labels = {'v86': 'V8.6 (Seagull)', 'v89': 'V8.9 (Pre-Seagull)', 'lkh': 'LKH'}
lw = {'v86': 1.2, 'v89': 1.5, 'lkh': 0.8}
styles = {'v86': '-', 'v89': '-', 'lkh': '--'}

for idx, inst in enumerate(data):
    ax = axes[idx]
    nodes = np.array(inst['nodes'])
    x, y = nodes[:, 0], nodes[:, 1]

    ax.scatter(x, y, c='#444444', s=12, zorder=1)

    for key in ['lkh', 'v86', 'v89']:
        if key in inst:
            tour = inst[key]['tour'] + [inst[key]['tour'][0]]  # close cycle
            dist = inst[key]['dist']
            err = (dist - inst['optimal']) / inst['optimal'] * 100
            tx = nodes[tour, 0]
            ty = nodes[tour, 1]
            ax.plot(tx, ty, color=colors[key], linewidth=lw[key],
                    linestyle=styles[key], label=f"{labels[key]}: {dist:.1f} ({err:+.2f}%)",
                    zorder=2)

    ax.set_title(f"{inst['name']} ({inst['n']} nodos) | Optimo: {inst['optimal']:.0f}",
                 fontsize=11, fontweight='bold')
    ax.legend(loc='upper right', fontsize=7, framealpha=0.7)
    ax.set_aspect('equal')
    ax.set_xticks([])
    ax.set_yticks([])
    for spine in ax.spines.values():
        spine.set_color('#333333')

fig.suptitle('Comparacion V8.6 (Seagull) vs V8.9 (Pre-Seagull) vs LKH',
             fontsize=14, fontweight='bold', y=0.98)
fig.patch.set_facecolor('#0a0a0a')
for ax in axes:
    ax.set_facecolor('#111111')
    ax.tick_params(colors='#888888')

plt.tight_layout()
fig.savefig('assets/comparison_v86_v89.png', dpi=150, bbox_inches='tight',
            facecolor='#0a0a0a')
print('✓ assets/comparison_v86_v89.png')
