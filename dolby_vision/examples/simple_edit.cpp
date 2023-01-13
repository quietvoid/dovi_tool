#include <fstream>
#include <iterator>
#include <vector>

extern "C"
{
#include "helpers.h"
}

int main(void) {
    std::ifstream input("../../assets/tests/fel_orig.bin", std::ios::binary);

    const std::vector<uint8_t> buf(
        (std::istreambuf_iterator<char>(input)),
        (std::istreambuf_iterator<char>()));

    input.close();

    DoviRpuOpaque *rpu = dovi_parse_unspec62_nalu(buf.data(), buf.size());
    const DoviRpuDataHeader *header = dovi_rpu_get_header(rpu);

    if (header) {
        int ret;

        ret = dovi_convert_rpu_with_mode(rpu, 2);
        if (ret < 0)
            goto fail;

        ret = dovi_rpu_set_active_area_offsets(rpu, 0, 0, 138, 138);
        if (ret < 0)
            goto fail;

        // Get new edited header, only if necessary
        dovi_rpu_free_header(header);

        header = dovi_rpu_get_header(rpu);
        ret = process_rpu_info(rpu, header);

        const DoviData *rpu_payload = dovi_write_unspec62_nalu(rpu);
        if (!rpu_payload)
            goto fail;
        
        // Do something with the edited payload
        dovi_data_free(rpu_payload);
    }

    process_rpu_info(rpu, header);

fail:
    if (header)
        dovi_rpu_free_header(header);

    dovi_rpu_free(rpu);
}
