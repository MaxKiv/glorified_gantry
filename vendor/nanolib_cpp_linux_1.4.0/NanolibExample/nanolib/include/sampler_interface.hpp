#pragma once

#include "device_handle.hpp"
#include "sampler_types.hpp"

namespace nlc {

/**
 * @brief Sampler interface
 */
class SamplerInterface {
public:
	virtual ~SamplerInterface() {
	}

public:

	/**
	 * @brief Configure a sampler
	 * @param [in] deviceHandle - handle of a device
	 * @param [in] samplerConfiguration - configuration to apply
	 * @return the result of the operation
	 */
	virtual ResultVoid configure(const DeviceHandle deviceHandle,
								 const SamplerConfiguration &samplerConfiguration)
		= 0;

	/**
	 * @brief Starts a sampler
	 * @param [in] deviceHandle - handle of a device
	 * @param [in] samplerNotify - optional notification, can be nullptr
	 * @param [in] applicationData - application specific data
	 * @return the result of the operation
	 */
	virtual ResultVoid start(const DeviceHandle deviceHandle, SamplerNotify *samplerNotify,
							 int64_t applicationData)
		= 0;

	/**
	 * @brief Getting sampled data
	 * @param [in] deviceHandle - handle of a device
	 * @return sampled data, which can be an empty array if samplerNotify is specified during start
	 */
	virtual ResultSampleDataArray getData(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Stops a sampler
	 * @param [in] deviceHandle - handle of a device
	 * @return the result of the operation
	 */
	virtual ResultVoid stop(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Getting the status of a sampler
	 * @return the status of a sampler
	 */
	virtual ResultSamplerState getState(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Getting the last error of a sampler
	 * @return last error of a sampler
	 */
	virtual ResultVoid getLastError(const DeviceHandle deviceHandle) = 0;
};

} // namespace nlc
