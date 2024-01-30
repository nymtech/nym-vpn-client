#include "stdafx.h"
#include "binarycomposer.h"
#include "memory.h"

namespace common
{

BinaryComposer::BinaryComposer(std::initializer_list<ConstBufferView> parts)
{
	m_offsets.reserve(parts.size());
	m_bufferSize = 0;

	for (const ConstBufferView &part: parts)
	{
		m_offsets.push_back(m_bufferSize);
		m_bufferSize += ::common::memory::AlignNative(part.size());
	}

	// Intentionally clear buffer after allocating it
	m_buffer = std::make_unique<uint8_t[]>(m_bufferSize);

	auto raw = m_buffer.get();
	auto index = 0;

	for (const ConstBufferView &part : parts)
	{
		auto offset = m_offsets[index];
		++index;

		memcpy(raw + offset, part.data(), part.size());
	}
}

const std::vector<size_t> &BinaryComposer::offsets() const
{
	return m_offsets;
}

uint8_t *BinaryComposer::buffer()
{
	return m_buffer.get();
}

Buffer BinaryComposer::acquire()
{
	return Buffer(m_buffer.release(), m_bufferSize);
}

size_t BinaryComposer::size() const
{
	return m_bufferSize;
}

}
