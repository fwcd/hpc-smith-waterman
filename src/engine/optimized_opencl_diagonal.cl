// Computes a diagonal slice of the Smith-Waterman matrices on the GPU.
// Mostly a translation of the inner loop from the optimized diagonal engine.
__kernel void smith_waterman_diagonal(
    uint width,
    uint offset,
    uint lower,
    uint lower_padding,
    uint previous_size,
    uint previous_previous_size,
    uint steps_since_in_bottom_part,
    __global uchar *database,
    __global uchar *query,
    __global short *h,
    __global short *e,
    __global short *f,
    __global uint *is,
    __global uint *js,
    __global uint *p
) {
    uint k = get_global_id(0);
    uint l = get_global_id(1);

    // Compute the 'actual'/'logical' position in the matrix.
    // We need this to index into the query/database sequence,
    // although we use our diagonal-major/cache-optimized
    // indexing scheme for the matrices instead.
    uint j = lower + l;
    uint i = k - j;

    // Compute indices of the neighboring cells.
    uint here = offset + lower_padding + l;
    uint above = here - previous_size + (steps_since_in_bottom_part > 0 ? 1 : 0);
    uint left = above - 1;
    uint above_left = left - previous_previous_size + (steps_since_in_bottom_part > 1 ? 1 : 0);

    // Write index mappings
    is[here] = i;
    js[here] = j;

    // Compute helper values
    short e_here = max(e[left] - G_EXT, h[left] - G_INIT);
    short f_here = max(f[above] - G_EXT, h[above] - G_INIT);

    e[here] = e_here;
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
