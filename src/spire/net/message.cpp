#include <spire/net/message.hpp>

#include <bit>
#include <cstring>

namespace spire::net {

void MessageHeader::serialize(const MessageHeader& source, std::span<std::byte, SIZE> target) {
    u16 body_size {source.body_size};
    if constexpr (std::endian::native == std::endian::little)
        body_size = std::byteswap(body_size);

    std::memcpy(target.data(), &body_size, sizeof(body_size));
}

MessageHeader MessageHeader::deserialize(const std::span<const std::byte, SIZE> source) {
    u16 body_size;
    std::memcpy(&body_size, source.data(), sizeof(body_size));
    if constexpr (std::endian::native == std::endian::little)
        body_size = std::byteswap(body_size);

    return MessageHeader {.body_size = body_size};
}

InMessage::InMessage(const Entity entity, std::vector<std::byte>&& data)
    : _entity {entity}, _data {std::move(data)} {}

OutMessage::OutMessage(const MessageHeader header) {
    _data.resize(sizeof(MessageHeader) + header.body_size);

    MessageHeader::serialize(header, std::span<std::byte, MessageHeader::SIZE> {_data.data(), MessageHeader::SIZE});
}
}