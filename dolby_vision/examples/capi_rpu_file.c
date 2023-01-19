#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>

#include "helpers.h"

int main(void) {
    char *path = "../../assets/hevc_tests/regular_rpu_mel.bin";
    int ret;

    const DoviRpuOpaqueList *rpus = dovi_parse_rpu_bin_file(path);
    if (rpus->error) {
        printf("%s\n", rpus->error);

        dovi_rpu_list_free(rpus);
        return 1;
    }

    printf("Parsed RPU file: %d frames\n", rpus->len);

    // All the RPUs are valid at this point
    DoviRpuOpaque *rpu = rpus->list[0];
    
    const DoviRpuDataHeader *header = dovi_rpu_get_header(rpu);

    // Process the RPU..
    ret = process_rpu_info(rpu, header);
    if (ret < 0) {
        const char *error = dovi_rpu_get_error(rpu);
        printf("%s\n", error);
    }

    // Free everything
    dovi_rpu_free_header(header);
    dovi_rpu_list_free(rpus);
}
