#pragma once

#include <vector>

#include "result_base.hpp"
#include "bus_hardware_id.hpp"
#include "device_id.hpp"
#include "device_handle.hpp"
#include "nlc_constants.hpp"
#include "od_types.hpp"

namespace nlc {

/**
 * @brief Instance of this class is returned in case the function returns void (aka "nothing").
 */
class ResultVoid : public Result {
public:
	ResultVoid() : Result() {
	}

	explicit ResultVoid(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultVoid(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultVoid(NlcErrorCode const &errCode, const uint32_t exErrCode,
						std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultVoid(Result const &result) : Result(result) {
	}

	virtual ~ResultVoid() {
	}
};

/**
 * @brief Instance of this class is returned in case the function returns an integer.
 */
class ResultInt : public Result {
public:
	ResultInt(int64_t result_) : Result(), result(result_) {
	}

	explicit ResultInt(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultInt(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultInt(NlcErrorCode const &errCode, const uint32_t exErrCode,
						std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultInt(Result const &result) : Result(result) {
	}

	virtual ~ResultInt() {
	}

	/**
	 * @brief Returns the integer result in case of a successful function call.
	 *
	 * @return int64_t
	 */
	int64_t getResult() const {
		return result;
	}

private:
	int64_t result{0};
};

/**
 * @brief Instance of this class is returned in case the function returns a string.
 */
class ResultString : public Result {
public:
	ResultString(std::string const &message, bool hasError_) {
		if (hasError_) {
			errorString = message;
			errorCode = NlcErrorCode::GeneralError;
		} else {
			errorCode = NlcErrorCode::Success;
			result = message;
		}
	}

	explicit ResultString(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultString(NlcErrorCode const &errCode, const uint32_t exErrCode,
					   std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultString(Result const &result) : Result(result) {
	}

	virtual ~ResultString() {
	}

	/**
	 * @brief Returns the string result in case of a successful function call.
	 *
	 * @return const std::string
	 */
	const std::string getResult() const {
		return result;
	}

private:
	std::string result{""};
};

/**
 * @brief Instance of this class is returned in case the function returns an array of integer.
 */
class ResultArrayInt : public Result {
public:
	ResultArrayInt(std::vector<int64_t> const &result_) : Result(), result(result_) {
	}

	explicit ResultArrayInt(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultArrayInt(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultArrayInt(NlcErrorCode const &errCode, const uint32_t exErrCode,
						  std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultArrayInt(Result const &result) : Result(result) {
	}

	virtual ~ResultArrayInt() {
	}

	/**
	 * @brief Returns the vector of integer in case of a successful function call.
	 *
	 * @return const std::vector<uint64_t>
	 */
	const std::vector<int64_t> getResult() const {
		return result;
	}

private:
	std::vector<int64_t> result;
};

/**
 * @brief Instance of this class is returned in case the function returns an array of bytes.
 */
class ResultArrayByte : public Result {
public:
	ResultArrayByte(std::vector<uint8_t> const &result_) : Result(), result(result_) {
	}

	explicit ResultArrayByte(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultArrayByte(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultArrayByte(NlcErrorCode const &errCode, const uint32_t exErrCode,
							std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultArrayByte(Result const &result) : Result(result) {
	}

	virtual ~ResultArrayByte() {
	}

	/**
	 * @brief Returns the vector of byte in case of a successful function call.
	 *
	 * @return const std::vector<uint8_t>
	 */
	const std::vector<uint8_t> getResult() const {
		return result;
	}

private:
	std::vector<uint8_t> result;
};


/**
 * @brief Instance of this class is returned in case the function returns an array of BusHardwareId.
 */
class ResultBusHwIds : public Result {
public:
	ResultBusHwIds(std::vector<BusHardwareId> const &result_) : Result(), result(result_) {
	}

	explicit ResultBusHwIds(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultBusHwIds(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultBusHwIds(NlcErrorCode const &errCode, const uint32_t exErrCode,
							 std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultBusHwIds(Result const &result) : Result(result) {
	}

	virtual ~ResultBusHwIds() {
	}

	/**
	 * @brief Returns the vector of BusHardwareId in case of a successful function call.
	 *
	 * @return const std::vector<BusHardwareId>
	 */
	const std::vector<BusHardwareId> getResult() const {
		return result;
	}

private:
	std::vector<BusHardwareId> result;
};

/**
 * @brief Instance of this class is returned in case the function returns a DeviceId.
 */
class ResultDeviceId : public Result {
public:
	ResultDeviceId(DeviceId const &result_) : Result(), result(result_) {
	}

	explicit ResultDeviceId(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultDeviceId(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultDeviceId(NlcErrorCode const &errCode, const uint32_t exErrCode,
							 std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultDeviceId(Result const &result) : Result(result) {
	}

	virtual ~ResultDeviceId() {
	}

	/**
	 * @brief Returns the vector of DeviceId in case of a successful function call.
	 *
	 * @return const std::vector<DeviceId>
	 */
	DeviceId getResult() const {
		return result;
	}

private:
	DeviceId result;
};


/**
 * @brief Instance of this class is returned in case the function returns an array of DeviceId.
 */
class ResultDeviceIds : public Result {
public:
	ResultDeviceIds(std::vector<DeviceId> const &result_) : Result(), result(result_) {
	}

	explicit ResultDeviceIds(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultDeviceIds(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultDeviceIds(NlcErrorCode const &errCode, const uint32_t exErrCode,
							std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultDeviceIds(Result const &result) : Result(result) {
	}

	virtual ~ResultDeviceIds() {
	}

	/**
	 * @brief Returns the vector of DeviceId in case of a successful function call.
	 *
	 * @return const std::vector<DeviceId>
	 */
	const std::vector<DeviceId> getResult() const {
		return result;
	}

private:
	std::vector<DeviceId> result;
};

/**
 * @brief Instance of this class is returned in case the function returns a DeviceHandle.
 */
class ResultDeviceHandle : public Result {
public:
	ResultDeviceHandle(DeviceHandle const &result_) : Result(), result(result_) {
	}

	explicit ResultDeviceHandle(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultDeviceHandle(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultDeviceHandle(NlcErrorCode const &errCode, const uint32_t exErrCode,
					   std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultDeviceHandle(Result const &result) : Result(result) {
	}

	virtual ~ResultDeviceHandle() {
	}

	/**
	 * @brief Returns the DeviceHandle result in case of a successful function call.
	 *
	 * @return DeviceHandle
	 */
	DeviceHandle getResult() const {
		return result;
	}

private:
	DeviceHandle result;
};


/**
 * @brief Instance of this class is returned in case the function returns a DeviceConnectionStateInfo.
 */
class ResultConnectionState : public Result {
public:
	ResultConnectionState(DeviceConnectionStateInfo const &result_) : Result(), result(result_) {
	}

	explicit ResultConnectionState(std::string const &errorString_) : Result(errorString_) {
	}

	explicit ResultConnectionState(NlcErrorCode const &errCode, std::string const &errorString_)
		: Result(errCode, errorString_) {
	}

	explicit ResultConnectionState(NlcErrorCode const &errCode, const uint32_t exErrCode,
								std::string const &errorString_)
		: Result(errCode, exErrCode, errorString_) {
	}

	explicit ResultConnectionState(Result const &result) : Result(result) {
	}

	virtual ~ResultConnectionState() {
	}

	/**
	 * @brief Returns the DeviceConnectionStateInfo in case of a successful function call.
	 *
	 * @return DeviceConnectionStateInfo
	 */
	DeviceConnectionStateInfo getResult() const {
		return result;
	}

private:
	DeviceConnectionStateInfo result;
};

} // namespace nlc
