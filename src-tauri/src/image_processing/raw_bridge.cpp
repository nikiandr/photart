#include "libraw/libraw.h"

#include <algorithm>
#include <cstdint>
#include <cstring>
#include <memory>

extern "C" {
struct PhotartRawMetadata {
    char make[64];
    char model[64];
    char lens[128];
    std::int64_t timestamp;
    float iso, aperture, shutter;
};

struct PhotartRawPixels {
    const unsigned char *data;
    std::uint32_t length, width, height;
};

void *photart_raw_open(const unsigned char *bytes, std::size_t length, std::uint64_t max_pixels,
                       int *status) noexcept {
    try {
        auto decoder = std::make_unique<LibRaw>();
        decoder->imgdata.rawparams.max_raw_memory_mb = 512;
        const int result = decoder->open_buffer(bytes, length);
        if (result != LIBRAW_SUCCESS) {
            *status = result == LIBRAW_FILE_UNSUPPORTED ? 1 : result == LIBRAW_TOO_BIG ? 2 : 3;
            return nullptr;
        }
        const auto &sizes = decoder->imgdata.sizes;
        if (!sizes.width || !sizes.height || !sizes.raw_width || !sizes.raw_height ||
            !decoder->imgdata.idata.raw_count) {
            *status = 1;
            return nullptr;
        }
        if (std::uint64_t(sizes.width) * sizes.height > max_pixels ||
            std::uint64_t(sizes.raw_width) * sizes.raw_height > max_pixels) {
            *status = 2;
            return nullptr;
        }
        return decoder.release();
    } catch (...) {
        *status = 3;
        return nullptr;
    }
}

void photart_raw_close(void *handle) noexcept {
    delete static_cast<LibRaw *>(handle);
}

void photart_raw_metadata(void *handle, PhotartRawMetadata *output) noexcept {
    const auto &data = static_cast<LibRaw *>(handle)->imgdata;
    std::memcpy(output->make, data.idata.make, sizeof(output->make));
    std::memcpy(output->model, data.idata.model, sizeof(output->model));
    const char *lens = data.lens.Lens[0] ? data.lens.Lens : data.lens.makernotes.Lens;
    std::memcpy(output->lens, lens, sizeof(output->lens));
    output->timestamp = static_cast<std::int64_t>(data.other.timestamp);
    output->iso = data.other.iso_speed;
    output->aperture = data.other.aperture;
    output->shutter = data.other.shutter;
}

void *photart_raw_pixels(void *handle, std::uint32_t render_goal, PhotartRawPixels *output) noexcept {
    try {
        auto &decoder = *static_cast<LibRaw *>(handle);
        libraw_processed_image_t *image = nullptr;
        if (!render_goal) {
            if (decoder.unpack_thumb() == LIBRAW_SUCCESS) image = decoder.dcraw_make_mem_thumb();
        } else {
            auto &data = decoder.imgdata;
            data.params.half_size = std::max(data.sizes.width, data.sizes.height) / 2 >= render_goal;
            data.params.use_camera_wb = 1;
            data.params.no_auto_bright = 1;
            if (decoder.unpack() == LIBRAW_SUCCESS && decoder.dcraw_process() == LIBRAW_SUCCESS)
                image = decoder.dcraw_make_mem_image();
        }
        if (!image) return nullptr;
        const bool supported = render_goal
            ? image->type == LIBRAW_IMAGE_BITMAP && image->colors == 3 && image->bits == 8
            : image->type == LIBRAW_IMAGE_JPEG;
        if (!supported) {
            LibRaw::dcraw_clear_mem(image);
            return nullptr;
        }
        *output = {image->data, image->data_size, image->width, image->height};
        return image;
    } catch (...) {
        return nullptr;
    }
}

void photart_raw_free_pixels(void *image) noexcept {
    LibRaw::dcraw_clear_mem(static_cast<libraw_processed_image_t *>(image));
}
}
