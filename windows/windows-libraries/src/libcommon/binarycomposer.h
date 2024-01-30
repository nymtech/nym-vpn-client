#pragma once

#include "buffer.h"
#include <initializer_list>
#include <memory>
#include <vector>

namespace common
{

class BinaryComposer
{
public:

	BinaryComposer(std::initializer_list<ConstBufferView> parts);

	const std::vector<size_t> &offsets() const;

	uint8_t *buffer();

	Buffer acquire();

	size_t size() const;

private:

	std::vector<size_t> m_offsets;
	std::unique_ptr<uint8_t[]> m_buffer;
	size_t m_bufferSize;
};

}
