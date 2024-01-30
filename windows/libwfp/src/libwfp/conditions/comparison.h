#pragma once

#include "icomparisonoperator.h"

namespace wfp::conditions {

//
// For usage when implementing conditions, internal storage of comparision spec
// after the comparision inheritance check has passed
//
class ComparisonSpecification
{
public:

	ComparisonSpecification(const IComparisonOperator &instance)
		: m_op(instance.op())
		, m_string(instance.toString())
	{
	}

	FWP_MATCH_TYPE op() const
	{
		return m_op;
	}

	std::wstring toString() const
	{
		return m_string;
	}

private:

	FWP_MATCH_TYPE m_op;
	std::wstring m_string;
};

struct CompareEq : public IStrictComparison, public IRelaxedComparison
{
	FWP_MATCH_TYPE op() const override
	{
		return FWP_MATCH_EQUAL;
	}

	std::wstring toString() const override
	{
		return L"=";
	}
};

struct CompareNeq : public IStrictComparison, public IRelaxedComparison
{
	FWP_MATCH_TYPE op() const override
	{
		return FWP_MATCH_NOT_EQUAL;
	}

	std::wstring toString() const override
	{
		return L"!=";
	}
};

struct CompareGt : public IRelaxedComparison
{
	FWP_MATCH_TYPE op() const override
	{
		return FWP_MATCH_GREATER;
	}

	std::wstring toString() const override
	{
		return L">";
	}
};

struct CompareGte : public IRelaxedComparison
{
	FWP_MATCH_TYPE op() const override
	{
		return FWP_MATCH_GREATER_OR_EQUAL;
	}

	std::wstring toString() const override
	{
		return L">=";
	}
};

struct CompareLt : public IRelaxedComparison
{
	FWP_MATCH_TYPE op() const override
	{
		return FWP_MATCH_LESS;
	}

	std::wstring toString() const override
	{
		return L"<";
	}
};

struct CompareLte : public IRelaxedComparison
{
	FWP_MATCH_TYPE op() const override
	{
		return FWP_MATCH_LESS_OR_EQUAL;
	}

	std::wstring toString() const override
	{
		return L"<=";
	}
};

}
