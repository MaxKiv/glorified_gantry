#pragma once
#include <string>
#include <cstdint>

#include "nlc_errorcode.hpp"

namespace nlc {

/**
 * @brief This class represents a return value.
 *
 * This class is in essence what other programming languages offer as an 'optional' data type.
 *
 * In case of a successful call the function hasError() returns false. The result value can be
 * acessed with the getResult() function.
 *
 * In case of a error the function hasError() returns true, the reason for the failure can be
 * read with the function getError().
 *
 */
class Result {
public:
	Result(std::string const &errorString_)
		: errorString(errorString_), errorCode(NlcErrorCode::GeneralError),
		  exErrorCode(0) {
	}

	Result(NlcErrorCode const &errCode, std::string const &errorString_)
		: errorString(errorString_), errorCode(errCode), exErrorCode(0) {
	}

	Result(NlcErrorCode const &errCode, const uint32_t exErrCode, std::string const &errorString_)
		: errorString(errorString_), errorCode(errCode), exErrorCode(exErrCode) {
	}

	explicit Result(Result const &result)
		: errorString(result.errorString),
		  errorCode(result.errorCode),
		  exErrorCode(result.exErrorCode) {
	}

	Result& operator=(const Result& other) = default;
	Result& operator=(Result&& other) = default;

	Result() : errorCode(NlcErrorCode::Success), exErrorCode(0) {
	}

	virtual ~Result() = 0;

	/**
	 * @brief Returns the success of the function call
	 *
	 * @return true  In case the function call failed, use the getError() function to obtain the
	 * the value.
	 * @return false In case the function call was successful, use the getResult() function to obtain
	 * reason.
	 */
	bool hasError() const {
		return errorCode != NlcErrorCode::Success;
	}

	/**
	 * @brief In case of an error result this function returns the reason
	 *
	 * @return const std::string
	 */
	const std::string getError() const {
		return errorString;		
	}

	NlcErrorCode getErrorCode() const {
		return errorCode;
	}

	uint32_t getExErrorCode() const {
		return exErrorCode;
	}

protected:
	std::string errorString;
	NlcErrorCode errorCode;
	uint32_t exErrorCode;
};

inline Result::~Result() {

}

} // namespace nlc
