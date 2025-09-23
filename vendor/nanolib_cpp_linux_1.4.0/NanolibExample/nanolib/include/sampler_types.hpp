#pragma once

#include <cstdint>
#include <string>
#include <vector>

#include "od_index.hpp"
#include "result.hpp"

namespace nlc {

/**
 *  @brief Sampler state
 */
enum class SamplerState {

	// Not yet configured
	Unconfigured,

	// Configured but not started
	Configured,

	// Configured and waiting for the start trigger
	Ready,

	// Running now
	Running,

	// Completed successfully
	Completed,

	// Finished due to an error
	Failed,

	// Cancelled from the application
	Cancelled
};

/**
 * @brief Result successor, with state of the sampler
 */
class ResultSamplerState : public Result {
public:
	ResultSamplerState(const SamplerState state) : result(state) {
	}

	ResultSamplerState(const std::string &errorDesc,
					   const NlcErrorCode errorCode = NlcErrorCode::GeneralError,
					   const uint32_t extendedErrorCode = 0)
		: Result(errorCode, extendedErrorCode, errorDesc), result(SamplerState::Unconfigured) {
	}

	ResultSamplerState(const ResultSamplerState &other) : Result(other), result(other.result) {
	}

	ResultSamplerState(const Result &result) : Result(result), result(SamplerState::Unconfigured) {
	}

	~ResultSamplerState() = default;

	/**
	 * @brief Returns the SamplerState in case of a successful function call.
	 *
	 * @return SamplerState
	 */
	SamplerState getResult() const {
		return result;
	}

private:
	SamplerState result;
};

/**
 * @brief Trigger condition
 */
enum class SamplerTriggerCondition : uint8_t {

	// Never
	TC_FALSE = 0x00,

	// Immediate
	TC_TRUE = 0x01,

	// Bit set
	// *trigger & (1 << value) != 0
	TC_SET = 0x10,

	// Bit clear
	// *trigger & (1 << value) == 0
	TC_CLEAR = 0x11,

	// Bit rising
	// (trigger[-1] & (1 << value) == 0) && (*trigger & (1 << value) != 0)
	TC_RISING_EDGE = 0x12,

	// Bit falling
	// (trigger[-1] & (1 << value) != 0) && (*trigger & (1 << value) == 0)
	TC_FALLING_EDGE = 0x13,

	// Bit changing
	// (trigger[-1] & (1 << value)) != (*trigger & (1 << value))
	TC_BIT_TOGGLE = 0x14,

	// *trigger > value
	TC_GREATER = 0x15,

	// *trigger >= value
	TC_GREATER_OR_EQUAL = 0x16,

	// *trigger < value
	TC_LESS = 0x17,

	// *trigger <= value
	TC_LESS_OR_EQUAL = 0x18,

	// *trigger == value
	TC_EQUAL = 0x19,

	// *trigger != value
	TC_NOT_EQUAL = 0x1A,

	// (value > 0)
	// ? (*trigger - trigger[-1] > value)
	// : (*trigger - trigger[-1] < value)
	TC_ONE_EDGE = 0x1B,

	// abs(trigger[-1] - *trigger) > abs(value)
	TC_MULTI_EDGE = 0x1C
};

/**
 *  @brief Sampler mode
 */
enum class SamplerMode : uint8_t {

	// (Single) oneshot execution
	Normal,

	// Starts again after it is finished
	// The trigger is checked before it starts collecting data for each collection.
	// SampleData::iterationNumber increases by one for each new iteration
	Repetitive,

	// Endless mode.
	// The duration should be set to 0.
	// The trigger is checked only once.
	// ONLY in software mode
	Continuous
};

/**
 * @brief Sampler trigger
 */
struct SamplerTrigger {

	/**
	 * @brief The trigger condition
	 */
	SamplerTriggerCondition condition;

	/**
	 * @brief OD address of the trigger
	 */
	OdIndex address;

	/**
	 * @brief Condition value or bit number. Bit numbering starts at zero.
	 */
	uint32_t value;

	SamplerTrigger() {
		condition = SamplerTriggerCondition::TC_FALSE;
		value = 0;
	}
};

/**
 *  @brief Sampler configuration
 */
struct SamplerConfiguration {

	/**
	 * @brief A version of the structure.
	 */
	uint32_t version;
	static constexpr size_t SAMPLER_CONFIGURATION_VERSION = 0x01000000;

	/**
	 * @brief Mode of the sampler
	 */
	SamplerMode mode;

	/**
	 * @brief Using software implementation
	 */
	bool usingSoftwareImplementation;

	/**
	 * @brief Using new FW sampler inerface implementation (FW >= v2400)
	 */
	bool usingNewFWSamplerImplementation;

	/**
	 * @brief Sampling period in milliseconds from 1..65535.
	 */
	uint16_t periodMilliseconds;

	/**
	 * @brief Duration in milliseconds.
	 */
	uint32_t durationMilliseconds;

	/**
	 * @brief Pre-trigger number of samplings
	 */
	uint16_t preTriggerNumberOfSamples;

	/**
	 * @brief Start trigger
	 */
	SamplerTrigger startTrigger;

	/**
	 * @brief Stop trigger
	 */
	SamplerTrigger stopTrigger;

	/**
	 * @brief Up to 12 OD addresses to track
	 */
	std::vector<OdIndex> trackedAddresses;
	static constexpr size_t MAX_TRACKED_ADDRESSES = 12;

	SamplerConfiguration() {

		version = SAMPLER_CONFIGURATION_VERSION;
		mode = SamplerMode::Normal;
		usingSoftwareImplementation = false;
		periodMilliseconds = 0;
		durationMilliseconds = 0;
		preTriggerNumberOfSamples = 0;
		usingNewFWSamplerImplementation = false;
	}

	/* 

		The start trigger is required, so it can't be TC_FALSE.
		Measurements start when the start trigger condition is met.
		In Repetitive mode, this condition is checked before the start of each iteration (curve).


		Duration is mandatory in firmware mode.
		In firmware mode, the maximum number of samplings is calculated according to the formula:

			numberOfSamples = 1 + (durationMilliseconds / periodMilliseconds)

		Values between 1 and (3048 / numberOfTrackedAddresses) are allowed, with a maximum of 12 buffers. 
		Each buffer has a capacity of 254 values. 
		Consecutive use of two or more buffers to store the values of a tracked address is allowed. 
		It is not possible to use the same buffer to store the values of more than one tracked address.


		Mode			Software		Duration		Stop Trigger		Behavior
		--------------------------------------------------------------------------------------------------------------------------------------------------------------
		Normal			No					0				No				ERROR [InvalidArguments] - The duration is required to calculate the number of samples.
		Normal			No					0				Yes				ERROR [InvalidArguments] - The duration is required to calculate the number of samples.
		Normal			No				>	0				No				The number of samples calculated according to the formula is taken.
		Normal			No				>	0				Yes				Sampling continues until the calculated number of samples is accumulated or until the stop trigger condition is met.
		Normal			Yes					0				No				ERROR [InvalidArguments] - No conditions to end sampling.
		Normal			Yes					0				Yes				Sampling continues until the stop trigger condition is met.
		Normal			Yes				>	0				No				Sampling continues until the set duration has elapsed.
		Normal			Yes				>	0				Yes				Sampling continues until the set duration elapses or until the stop trigger condition is met.

		Repetitive		No					0				No				ERROR [InvalidArguments] - The duration is required to calculate the number of samples.
		Repetitive		No					0				Yes				ERROR [InvalidArguments] - The duration is required to calculate the number of samples.
		Repetitive		No				>	0				No				The number of samples calculated according to the formula is taken. Then sampling starts again. The start trigger condition is checked before each iteration, and iterationNumber is incremented after each iteration.
		Repetitive		No				>	0				Yes				Each iteration continues until the calculated number of samples is accumulated or until the stop trigger condition is met. Sampling then starts again, with the start condition checked and the iteration number incremented.
		Repetitive		Yes					0				No				ERROR [InvalidArguments] - No conditions to end sampling.
		Repetitive		Yes					0				Yes				Each iteration continues until the stop trigger condition is met. Sampling then starts again, with the start condition checked and the iteration number incremented.
		Repetitive		Yes				>	0				No				Each iteration continues until the set duration expires. Then sampling starts again.
		Repetitive		Yes				>	0				Yes				Each iteration continues until the set duration elapses or until the stop trigger condition is met. Then sampling starts again.

		Continuous		No					0				No				ERROR [InvalidArguments] - Continuous sampling is not supported in firmware mode.
		Continuous		No					0				Yes				ERROR [InvalidArguments] - Continuous sampling is not supported in firmware mode.
		Continuous		No				>	0				No				ERROR [InvalidArguments] - Continuous sampling is not supported in firmware mode.
		Continuous		No				>	0				Yes				ERROR [InvalidArguments] - Continuous sampling is not supported in firmware mode.
		Continuous		Yes					0				No				Sampling is carried out to infinity, i.e. until the process is stopped.
		Continuous		Yes					0				Yes				ERROR [InvalidArguments] - Duration and stop condition are incompatible with Continuous mode.
		Continuous		Yes				>	0				No				ERROR [InvalidArguments] - Duration and stop condition are incompatible with Continuous mode.
		Continuous		Yes				>	0				Yes				ERROR [InvalidArguments] - Duration and stop condition are incompatible with Continuous mode.	

	*/
};

/**
 * @brief Sampled value
 */
struct SampledValue {

	/**
	 * @brief Value of a tracked OD address
	 */
	int64_t value;

	/**
	 * @brief Collection time in milliseconds, to the beginning of the execution
	 */
	uint64_t collectTimeMsec;
};

/**
 * @brief Sampled data
 */
struct SampleData {

	/**
	 * @brief Iteration number. It starts at 0 and only increases in Repetitive mode.
	 */
	uint64_t iterationNumber;

	/**
	 * @brief Array of sampled values.
	 */
	std::vector<SampledValue> sampledValues;
};

/**
 * @brief Result successor, with an array of sampled values
 */
class ResultSampleDataArray : public Result {
public:

	ResultSampleDataArray() = default;

	ResultSampleDataArray(const std::vector<SampleData> &dataArray) : sampleDataArray(dataArray) {
	}

	ResultSampleDataArray(const std::string &errorDesc,
						  const NlcErrorCode errorCode = NlcErrorCode::GeneralError,
						  const uint32_t extendedErrorCode = 0)
		: Result(errorCode, extendedErrorCode, errorDesc) {
	}

	ResultSampleDataArray(const ResultSampleDataArray &other)
		: Result(other), sampleDataArray(other.getResult()) {
	}

	ResultSampleDataArray(const Result &result) : Result(result) {
	}

	~ResultSampleDataArray() = default;

	const std::vector<SampleData> &getResult() const {
		return sampleDataArray;
	}

private:
	std::vector<SampleData> sampleDataArray;
};

/**
 * @brief Sampler notification
 */
class SamplerNotify {
public:
	virtual ~SamplerNotify() {
	}

	/**
	 * @brief Notification entry
	 * @param [in] lastError		- the last error occurred during the sampling
	 * @param [in] samplerState		- sampler status at the time of notification
	 * @param [in] sampleDatas		- array of sampled data
	 * @param [in] applicationData	- application specific data
	 */
	virtual void notify(const ResultVoid &lastError, const SamplerState samplerState,
						const std::vector<SampleData> &sampleDatas, int64_t applicationData)
		= 0;
};

} // namespace nlc
