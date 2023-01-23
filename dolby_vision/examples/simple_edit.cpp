#include <chrono>
#include <iostream>
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

    auto start = std::chrono::high_resolution_clock::now();

    DoviRpuOpaque *rpu = dovi_parse_unspec62_nalu(buf.data(), buf.size());
    const DoviRpuDataHeader *header = dovi_rpu_get_header(rpu);

    // Only converts profile 7 as they are guaranteed to be HDR10 base
    if (header && header->guessed_profile == 7) {
        // Convert the base to 8.1 compatible
        // Also handles removing mapping for FEL
        int ret = dovi_convert_rpu_with_mode(rpu, 2);

        // Final video has letterboxing completely cropped
        ret = dovi_rpu_set_active_area_offsets(rpu, 0, 0, 0, 0);

        const DoviData *rpu_payload = dovi_write_unspec62_nalu(rpu);
        
        // Do something with the edited payload
        dovi_data_free(rpu_payload);
    }

    if (header)
        dovi_rpu_free_header(header);

    dovi_rpu_free(rpu);

    auto end = std::chrono::high_resolution_clock::now();
    std::cout << std::chrono::duration_cast<std::chrono::microseconds>(end - start).count() << " μs";
}
