#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>

#include "helpers.h"

int main(void) {
    char *path = "../../assets/tests/cmv40_full_rpu.bin";
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
        return 1;
    }

    // Process the RPU..
    ret = process_rpu_info(rpu, header);
    if (ret < 0) {
        const char *error = dovi_rpu_get_error(rpu);
        printf("%s\n", error);
    }

    // Free everything
    dovi_rpu_free_header(header);
    dovi_rpu_free(rpu);
}
