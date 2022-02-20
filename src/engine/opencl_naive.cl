// Computes the Smith-Waterman matrices on the GPU.
// Mostly a translation of the naive engine.
__kernel void smith_waterman_naive(
    uint n,
    uint m,
    __global uchar *database,
    __global uchar *query,
    __global short *h,
    __global short *f,
    __global uint *p
) {
    // TODO: Use local memory for the intermediate matrices

    uint width = m + 1;

    for (uint i = 0; i < n; i++) {
        short e_here = 0;

        for (uint j = 0; j < m; j++) {
            // Compute indices of the neighboring cells
            uint here = i * width + j;
            uint above = (i - 1) * width + j;
            uint left = i * width + j - 1;
            uint above_left = (i - 1) * width + j - 1;

            // Compute helper values
            e_here = max(e_here - G_EXT, h[left] - G_INIT);
            short f_here = max(f[above] - G_EXT, h[above] - G_INIT);

            f[here] = f_here;

            // Compute value and remember the index the maximum came from
            // (we need this later for the traceback phase)
            short from_above_left = h[above_left] + (database[i - 1] == query[j - 1] ? WEIGHT_IF_EQ : -WEIGHT_IF_EQ);
            uint max_origin = 0;
            short max_value = 0;
            
            if (from_above_left >= max_value) {
                max_origin = above_left;
                max_value = from_above_left;
            }

            if (e_here >= max_value) {
                max_origin = left;
                max_value = e_here;
            }

            if (f_here >= max_value) {
                max_origin = above;
                max_value = f_here;
            }

            h[here] = max_value;
            p[here] = max_origin;
        }
    }
}
