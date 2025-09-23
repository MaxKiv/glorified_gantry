#pragma once

#include "object_dictionary.hpp"

namespace nlc {

/**
 * @brief Instance of this class is returned in case the function returns a ObjectDictionary.
 */
 class ResultObjectDictionary : public Result {
 public:
	 ResultObjectDictionary(nlc::ObjectDictionary const &result_) : Result(), result(result_) {
	}

	explicit ResultObjectDictionary(std::string const &errorString_)
		: Result(errorString_), result(invalidObject) {
	}

	explicit ResultObjectDictionary(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_), result(invalidObject) {
	}

	explicit ResultObjectDictionary(NlcErrorCode const &errCode, const uint32_t exErrCode,
								  std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_), result(invalidObject) {
	}

	explicit ResultObjectDictionary(Result const &result) : Result(result), result(invalidObject) {
	}

	virtual ~ResultObjectDictionary() {
	}

	/**
	 * @brief Returns a vector of ObjectDictionary in case of a successful function call.
	 *
	 * @return const std::vector<ObjectDictionary>
	 */
	const nlc::ObjectDictionary &getResult() const {
		return result;
	}

 protected:
	static inline const nlc::ObjectDictionary invalidObject;
	nlc::ObjectDictionary const &result;
 };

} // namespace nlc
