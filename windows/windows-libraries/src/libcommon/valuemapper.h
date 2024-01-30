#pragma once

#include "error.h"
#include <algorithm>
#include <utility>
#include <vector>
#include <initializer_list>
#include <optional>
#include <sstream>

namespace common
{

struct ValueMapper
{
	template<typename T, typename U, std::size_t S>
	static std::optional<U> TryMap(T t, const std::pair<T, U>(&dictionary)[S])
	{
		for (const auto &entry : dictionary)
		{
			if (t == entry.first)
			{
				return std::make_optional(entry.second);
			}
		}
		return std::nullopt;
	}

	template<typename T, typename U, std::size_t S>
	static U Map(T t, const std::pair<T, U> (&dictionary)[S])
	{
		auto result = TryMap(t, dictionary);
		if (result.has_value())
		{
			return result.value();
		}

		std::stringstream ss;
		ss << "Could not map between values: "
			<< typeid(T).name() << " -> " << typeid(U).name();

		THROW_ERROR(ss.str().c_str());
	}

};

}
