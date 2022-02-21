# Smith Waterman in Rust for HPC

A GPU-accelerated implementation of the Smith-Waterman algorithm for finding optimal local sequence aligments in Rust.

## Usage

> Note: If you want to build the program from source rather than use a prebuilt binary, substitute `cargo run --release --` for every occurrence of `hpc-smith-waterman` in the following commands.

The program includes two main modes: `run` and `bench`.

### Run Mode

In the first mode, the program will run every engine on a single database/query pair. E.g.

```
hpc-smith-waterman run
```

will run the algorithm on the default pair of sequences (`TGTTACGG` and `GGTTGACTA`). You can, however, also specify a custom pair of sequences

```
hpc-smith-waterman run GATT ATBTAG
```

will run the algorithm on the given pair (`GATT` and `ATBAG`).

### Bench Mode

> Note: To use bench mode, you need to either make sure that a dataset exists at `data/uniprot_sprot.fasta` from your cwd (you can download this dataset with the script `scripts/download-dataset`) or point to a custom FASTA-dataset with `--path`.

In the second mode, the program will read a dataset and then compare the first sequence to all of the remaining sequences, again using each engine. During this, the elapsed time and the Giga-CUPS (Cell Operations Per Second) will be recorded.

The simplest way to invoke this mode is to not pass any arguments:

```
hpc-smith-waterman bench
```

This will use the aforementioned dataset (`data/unipro_sprot.fasta`) and run every engine. If you only wish to run a subset of engines, you can pass the engines you wish to run as arguments. The following engines are supported:

| Flag | Description |
| ---- | ----------- |
| `--naive` | A naive CPU engine |
| `--diagonal` | A CPU engine that parallelizes over diagonals |
| `--opencl-diagonal` | A GPU engine that parallelizes over diagonals |
| `--opencl-diagonal` | A GPU engine that parallelizes over diagonals |
| `--optimized-diagonal` | A CPU engine that parallelizes over diagonals and uses a cache-optimized (diagonal-major) matrix layout |
| `--optimized-opencl-diagonal` | A GPU engine that parallelizes over diagonals and uses a cache-optimized (diagonal-major) matrix layout |

For example, if you wish to bench the naive engine and the OpenCL diagonal engine, you could invoke the program as follows:

```
hpc-smith-waterman bench --naive --opencl-diagonal
```

You can customize the maximum number of query sequenced benchmarked against using `--number` aka. `-n`:

```
hpc-smith-waterman bench -n 2000
```

If you wish, you can also let each query sequence benchmarked against be repeated/cycled a certain number of times using `--repeat` aka. `-r` to make it longer (useful for benchmarking the performance of very long sequences). For example, to bench against 50 sequences, each repeated/cycled 10 times in length, run

```
hpc-smith-waterman bench -n 50 -r 10
```

If you have multiple GPUs installed, you can choose the GPU for OpenCL using `--gpu-index` (the default is 0), e.g. like this:

```
hpc-smith-waterman --gpu-index 1 bench --opencl-diagonal
```

## Performance Considerations

While the benchmarks already parallelize over the examples using CPU threads, there are some observations to keep in mind:

- The GPU engines generally only outperform the CPU engines on large sequences (since those let us parallelize the kernel well due to lots of diagonals)
- Additionally, there is overhead to using OpenCL (e.g. configuring kernels, queueing them, etc.), which makes the CPU variants often faster when benchmarking lots of short sequences
- The naive CPU variant is already pretty fast due to good cache coherency (we iterate the matrix in a natural way, the inner loop visits adjacent elements)

## Example Results

Example benchmark results on the Apple M1 Pro:

### Lots of short-ish sequences

```
$ hpc-smith-waterman bench -n 10000
┌──────────────────────────┐
│ Naive (CPU) (sequential) │
└──────────────────────────┘
Elapsed: 9.79s
Giga-CUPS: 0.38
Pairs: 10000
┌────────────────────────┐
│ Naive (CPU) (parallel) │
└────────────────────────┘
Elapsed: 1.59s
Giga-CUPS: 2.34
Pairs: 10000
┌───────────────────────────┐
│ Diagonal (CPU) (parallel) │
└───────────────────────────┘
Elapsed: 4.79s
Giga-CUPS: 0.78
Pairs: 10000
┌─────────────────────────────────────┐
│ Optimized Diagonal (CPU) (parallel) │
└─────────────────────────────────────┘
Elapsed: 3.38s
Giga-CUPS: 1.10
Pairs: 10000
┌────────────────────────────────────────────────┐
│ OpenCL Diagonal (GPU: Apple M1 Pro) (parallel) │
└────────────────────────────────────────────────┘
Elapsed: 14.75s
Giga-CUPS: 0.25
Pairs: 10000
┌──────────────────────────────────────────────────────────┐
│ Optimized OpenCL Diagonal (GPU: Apple M1 Pro) (parallel) │
└──────────────────────────────────────────────────────────┘
Elapsed: 19.68s
Giga-CUPS: 0.19
Pairs: 10000
```

Observation: CPU variants outperform GPU variants by quite a bit.

### Few, very large sequences

> We exclude the naive engine since it's too slow.

```
$ hpc-smith-waterman bench --diagonal --opencl-diagonal --optimized-diagonal --optimized-opencl-diagonal -n 5 -r 36
┌───────────────────────────┐
│ Diagonal (CPU) (parallel) │
└───────────────────────────┘
Elapsed: 10.71s
Giga-CUPS: 0.18
Pairs: 5
┌─────────────────────────────────────┐
│ Optimized Diagonal (CPU) (parallel) │
└─────────────────────────────────────┘
Elapsed: 4.90s
Giga-CUPS: 0.39
Pairs: 5
┌────────────────────────────────────────────────┐
│ OpenCL Diagonal (GPU: Apple M1 Pro) (parallel) │
└────────────────────────────────────────────────┘
Elapsed: 7.46s
Giga-CUPS: 0.25
Pairs: 5
┌──────────────────────────────────────────────────────────┐
│ Optimized OpenCL Diagonal (GPU: Apple M1 Pro) (parallel) │
└──────────────────────────────────────────────────────────┘
Elapsed: 2.14s
Giga-CUPS: 0.89
Pairs: 5
```

Observation: The GPU is good at crunching large matrices with lots of diagonal in parallel.
