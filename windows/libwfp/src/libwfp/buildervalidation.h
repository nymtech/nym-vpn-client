#pragma once

namespace wfp
{

//
// Validation levels employed by builders before building an artifact.
//
enum class BuilderValidation
{
	// Ensure optional values are set if this is considered good practice.
	Extra,

	// Perform the bare minimum validation.
	OnlyCritical,

	// No validation
	Off
};

}
