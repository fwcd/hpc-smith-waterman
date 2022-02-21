// Computes a diagonal slice of the Smith-Waterman matrices on the GPU.
// Mostly a translation of the inner loop from the diagonal engine.
__kernel void smith_waterman_diagonal(
    uint width,
    __global uchar *database,
    __global uchar *query,
    __global short *h,
    __global short *e,
    __global short *f,
    __global uint *p
) {
    uint k = get_global_id(0);
    uint j = get_global_id(1);
    uint i = k - j;

    // TODO
}
