#pragma once

#include <memory>
#include <cstdint>

namespace common
{

struct IBuffer
{
	virtual ~IBuffer() = 0
	{
	}

	virtual uint8_t *data() const = 0;
	virtual size_t size() const = 0;
};

class Buffer : public IBuffer
{
public:

	Buffer() : m_size(0)
	{
	}

	Buffer(size_t s)
		: m_data(std::make_unique<uint8_t[]>(s))
		, m_size(s)
	{
	}

	Buffer(void *d, size_t s)
		: m_data(reinterpret_cast<uint8_t *>(d))
		, m_size(s)
	{
	}

	Buffer(Buffer &&rhs) = default;
	Buffer &operator=(Buffer &&rhs) = default;

	uint8_t *data() const override
	{
		return m_data.get();
	}

	size_t size() const override
	{
		return m_size;
	}

private:

	Buffer(const Buffer &rhs);
	Buffer &operator=(const Buffer &rhs);

	std::unique_ptr<uint8_t[]> m_data;
	size_t m_size;
};

class BufferView : public IBuffer
{
public:

	BufferView(void *d, size_t s)
		: m_data(reinterpret_cast<uint8_t *>(d))
		, m_size(s)
	{
	}

	uint8_t *data() const override
	{
		return m_data;
	}

	size_t size() const override
	{
		return m_size;
	}

private:

	uint8_t *m_data;
	size_t m_size;
};

class ConstBufferView
{
public:

	ConstBufferView(const void *d, size_t s)
		: m_data(reinterpret_cast<const uint8_t *>(d))
		, m_size(s)
	{
	}

	const uint8_t *data() const
	{
		return m_data;
	}

	size_t size() const
	{
		return m_size;
	}

private:

	const uint8_t *m_data;
	size_t m_size;
};

}
