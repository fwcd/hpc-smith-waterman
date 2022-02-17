// Computes a diagonal slice of the Smith-Waterman matrices on the GPU.
// Mostly a translation of the inner loop from the diagonal engine.
__kernel void smith_waterman_diagonal(
    int k,
    int width,
    int lower,
    __global uchar *database,
    __global uchar *query,
    __global int *h,
    __global int *e,
    __global int *f,
    __global int *p
) {
    int j = lower + get_global_id(0);
    int i = k - j;

    // Compute indices of the neighboring cells
    int here = i * width + j;
    int above = (i - 1) * width + j;
    int left = i * width + j - 1;
    int above_left = (i - 1) * width + j - 1;

    // Compute helper values
    int e_here = max(e[left] - G_EXT, h[left] - G_INIT);
    int f_here = max(f[above] - G_EXT, h[above] - G_INIT);

    e[here] = e_here;
    f[here] = f_here;

    // Compute value and remember the index the maximum came from
    // (we need this later for the traceback phase)
    int from_above_left = h[above_left] + (database[i - 1] == query[j - 1] ? WEIGHT_IF_EQ : -WEIGHT_IF_EQ);
    int max_origin = 0;
    int max_value = 0;
    
    if (from_above_left > max_value) {
        max_origin = above_left;
        max_value = from_above_left;
    }

    if (e_here > max_value) {
        max_origin = left;
        max_value = e_here;
    }

    if (f_here > max_value) {
        max_origin = above;
        max_value = f_here;
    }

    h[here] = max_value;
    p[here] = max_origin;
}