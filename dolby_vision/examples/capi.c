#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>

#include <libdovi/rpu_parser.h>

int do_something(DoviRpuOpaque *rpu, const DoviRpuDataHeader *header);
int process_rpu_data_mapping(DoviRpuOpaque *rpu, const DoviRpuDataHeader *header);
int process_dm_metadata(DoviRpuOpaque *rpu, const DoviRpuDataHeader *header);

const uint8_t* read_rpu_file(char *path, size_t *len);

int main(void) {
    char *path = "../../assets/tests/data_before_crc32.bin";
    int ret;

    size_t length;
    const uint8_t *buf = read_rpu_file(path, &length);

    DoviRpuOpaque *rpu = dovi_parse_unspec62_nalu(buf, length);
    free((void *) buf);
    
    // The RPU header is always present
    const DoviRpuDataHeader *header = dovi_rpu_get_header(rpu);
    if (!header) {
        const char *error = dovi_rpu_get_error(rpu);
        printf("%s\n", error);

        dovi_rpu_free(rpu);
        return 0;
    }

    // Process the RPU..
    ret = do_something(rpu, header);

    if (ret < 0) {
        const char *error = dovi_rpu_get_error(rpu);
        printf("%s\n", error);
    }

    // Free everything
    dovi_rpu_free_header(header);
    dovi_rpu_free(rpu);
}

int do_something(DoviRpuOpaque *rpu, const DoviRpuDataHeader *header) {
    const char *error;
    int ret;

    if (header->rpu_type != 2)
        return 0;

    // We have new rpu_data_mapping metadata
    if (!header->use_prev_vdr_rpu_flag) {
        ret = process_rpu_data_mapping(rpu, header);
    }

    // We have display management metadata
    if (header->vdr_dm_metadata_present_flag) {
        ret = process_dm_metadata(rpu, header);
    }

    // Convert FEL to MEL
    ret = dovi_convert_rpu_with_mode(rpu, 1);
    if (ret < 0) {
        return -1;
    }

    const DoviData *data = dovi_write_unspec62_nalu(rpu);
    if (!data) {
        return -1;
    }

    // Do something with the encoded RPU..

    // Free the encoded data when we're done
    dovi_data_free(data);

    return 0;
}

int process_rpu_data_mapping(DoviRpuOpaque *rpu, const DoviRpuDataHeader *header) {
    const DoviRpuDataMapping *rpu_data_mapping = dovi_rpu_get_data_mapping(rpu);
    if (!rpu_data_mapping)
        return -1;

    printf("vdr_rpu_data_mapping()\n");

    // Use the rpu_data_mapping metadata..
    for (int cmp = 0; cmp < DoviNUM_COMPONENTS; cmp++) {
        printf("  cmp %d\n", cmp);

        // 1D buffer example
        printf("    num_pivots: %d\n", header->num_pivots_minus_2[cmp] + 2);
        printf("      values: [", cmp);

        const uint64_t *buf = header->pred_pivot_value[cmp].data;
        for (int  i= 0; i < header->pred_pivot_value[cmp].len; i++) {
            printf(" %d", buf[i]);
        }
        printf(" ]\n");

        // 2D buffer example
        const DoviU64Data2D poly_coef = rpu_data_mapping->poly_coef[cmp];
        printf("    poly_coefs\n");

        for (int i = 0; i < poly_coef.len; i++) {
            const DoviU64Data *dovi_data = poly_coef.list[i];
            printf("      poly_coef[%d], len: %d, values: [", i, dovi_data->len);

            const uint64_t *buf = dovi_data->data;
            for (int j = 0; j < dovi_data->len; j++) {
                printf(" %d", buf[j]);
            }

            printf(" ]\n");
        }

        // 3D buffer example
        const DoviU64Data3D mmr_coef = rpu_data_mapping->mmr_coef[cmp];
        printf("    mmr_coefs, len: %d\n", mmr_coef.len);

        for (int i = 0; i < mmr_coef.len; i++) {
            const DoviU64Data2D *dovi_data_2d = mmr_coef.list[i];
            printf("      mmr_coef[%d], len: %d, values: [\n", i, dovi_data_2d->len);

            for (int j = 0; j < dovi_data_2d->len; j++) {
                const DoviU64Data *dovi_data = dovi_data_2d->list[j];
                printf("        mmr_coef[%d][%d], len: %d, values: [", i, j, dovi_data->len);
                
                const uint64_t *buf = dovi_data->data;
                for (int k = 0; k < dovi_data->len; k++) {
                    printf(" %d", buf[k]);
                }

                printf(" ]\n");
            }

            printf("      ]\n");
        }
    }

    dovi_rpu_free_data_mapping(rpu_data_mapping);

    // We have NLQ metadata (and therefore an enhancement layer)
    if (header->nlq_method_idc != -1) {
        const DoviRpuDataNlq *rpu_data_nlq = dovi_rpu_get_data_nlq(rpu);
        if (!rpu_data_nlq)
            return -1;

        printf("vdr_rpu_data_nlq()\n");

        // Do something with the NLQ data..
        const DoviU64Data2D vdr_in_max = rpu_data_nlq->vdr_in_max;
        printf("  NLQ data\n  vdr_in_max, len: %d\n", vdr_in_max.len);

        for (int i = 0; i < vdr_in_max.len; i++) {
            // This buffer is always size 3
            const DoviU64Data *dovi_data = vdr_in_max.list[i];

            printf("    vdr_in_max[%d], len: %d, values: [%d, %d, %d]\n", i,
                dovi_data->len, dovi_data->data[0], dovi_data->data[1], dovi_data->data[2]);
        }

        dovi_rpu_free_data_nlq(rpu_data_nlq);
    }

    return 0;
}

int process_dm_metadata(DoviRpuOpaque *rpu, const DoviRpuDataHeader *header) {
    const DoviVdrDmData *dm_data = dovi_rpu_get_vdr_dm_data(rpu);
    if (!dm_data)
        return -1;

    printf("vdr_dm_data_payload()\n");

    // Do something with the DM metadata..
    printf("  Mastering display PQ codes: min %.6f max %.6f\n", dm_data->source_min_pq / 4095.0,
           dm_data->source_max_pq / 4095.0);

    // We have the frame stats
    if (dm_data->st2094_10_metadata.level1) {
        const DoviExtMetadataBlockLevel1 *meta = dm_data->st2094_10_metadata.level1;

        // Values are PQ encoded in 12 bit, from 0 to 4095
        printf("  L1 Frame brightness: min %.6f, max %.6f, avg %.6f\n", meta->min_pq / 4095.0,
               meta->max_pq / 4095.0, meta->avg_pq / 4095.0);
    }

    // We have creative trims
    if (dm_data->st2094_10_metadata.level2.len > 0) {
        const DoviLevel2BlockList blocks = dm_data->st2094_10_metadata.level2; 
        printf("  L2 Creative trims, targets: %d\n", dm_data->st2094_10_metadata.level2.len);

        for (int i = 0; i < dm_data->st2094_10_metadata.level2.len; i++) {
            const DoviExtMetadataBlockLevel2 *meta = blocks.list[i];

            printf("    target display brightness PQ code: %.6f\n", meta->target_max_pq / 4095.0);

            // Trim values are from 0 to 4095
            printf("      trim_slope: %d, trim_offset: %d, trim_power: %d\n", meta->trim_slope,
                   meta->trim_offset, meta->trim_power);
            printf("      trim_chroma_weight: %d, trim_saturation_gain: %d, ms_weight: %d\n",
                   meta->trim_chroma_weight, meta->trim_saturation_gain, meta->ms_weight);
        }
    }

    // We have active area metadata
    if (dm_data->st2094_10_metadata.level5) {
        const DoviExtMetadataBlockLevel5 *meta = dm_data->st2094_10_metadata.level5;

        printf("  L5 Active area offsets: top %d, bottom %d, left %d, right %d\n",
               meta->active_area_top_offset, meta->active_area_bottom_offset,
               meta->active_area_left_offset, meta->active_area_right_offset);
    }

    // We have fallback HDR10 metadata
    if (dm_data->st2094_10_metadata.level6) {
        const DoviExtMetadataBlockLevel6 *meta = dm_data->st2094_10_metadata.level6;

        printf("  L6 Mastering display: min %.4f, max %d\n",
               meta->min_display_mastering_luminance / 10000.0,
               meta->max_display_mastering_luminance);

        printf("    MaxCLL %d, MaxFALL %d\n", meta->max_content_light_level,
               meta->max_frame_average_light_level);
    }

    dovi_rpu_free_vdr_dm_data(dm_data);
}

const uint8_t* read_rpu_file(char *path, size_t *len) {
    FILE *fileptr;
    uint8_t *buffer;

    fileptr = fopen(path, "rb");
    fseek(fileptr, 0, SEEK_END);
    *len = (size_t) ftell(fileptr);
    rewind(fileptr);

    size_t size = *len * sizeof(uint8_t);

    buffer = (uint8_t *) malloc(size);
    fread(buffer, *len, 1, fileptr);
    fclose(fileptr);

    return buffer;
}
