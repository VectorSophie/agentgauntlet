# Dataset Import Scripts

These scripts populate `dataset/seeds/` from three open security datasets.

## Sources

### AdvBench (direct import)
- **Repo**: https://github.com/llm-attacks/llm-attacks
- **License**: MIT
- **Data**: 520 harmful-behavior goals from `data/advbench/harmful_behaviors.csv`
- **How used**: Each `goal` becomes a multi-turn prompt injection seed
- **Script**: `import_advbench.py` — downloads the CSV directly, no auth required

### ToolBench (taxonomy-derived)
- **Repo**: https://github.com/OpenBMB/ToolBench
- **Full data**: https://huggingface.co/datasets/ToolBench/ToolBench (requires HuggingFace auth)
- **License**: Apache 2.0
- **How used**: Seeds are derived from ToolBench's real-world API taxonomy across
  file-system, network, system, database, scheduling, and auth categories
- **Script**: `import_toolbench.py` — generates seeds from taxonomy; see script docstring
  for instructions to use the full HuggingFace dataset

### MemoryAgentBench (methodology-derived)
- **Repo**: https://github.com/HazyResearch/memory-agent-bench
- **Paper**: https://arxiv.org/abs/2402.04236
- **License**: Apache 2.0
- **How used**: Adversarial adaptations of MAB's four task types:
  key-value recall, temporal reasoning, multi-hop memory, associative memory
- **Script**: `import_memoryagentbench.py` — generates adversarial variants of the MAB taxonomy

## Running

```bash
cd dataset/import
python import_advbench.py
python import_toolbench.py
python import_memoryagentbench.py
```

Then regenerate scenarios:

```bash
cd dataset/generator
python generate_scenarios.py --count-per-category 30 --out ../generated
```
