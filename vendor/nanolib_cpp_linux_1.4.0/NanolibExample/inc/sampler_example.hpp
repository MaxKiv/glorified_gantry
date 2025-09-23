/*
* Nanotec Nanolib example
* Copyright (C) Nanotec GmbH & Co. KG - All Rights Reserved
*
* This product includes software developed by the
* Nanotec GmbH & Co. KG (http://www.nanotec.com/).
*
* The Nanolib interface headers and the examples source code provided are 
* licensed under the Creative Commons Attribution 4.0 Internaltional License. 
* To view a copy of this license, 
* visit https://creativecommons.org/licenses/by/4.0/ or send a letter to 
* Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.
*
* The parts of the library provided in binary format are licensed under 
* the Creative Commons Attribution-NoDerivatives 4.0 International License. 
* To view a copy of this license, 
* visit http://creativecommons.org/licenses/by-nd/4.0/ or send a letter to 
* Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
*
* @file   sampler_example.hpp
*
* @brief  Declaration of sampler example class
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#pragma once
#include "menu_utils.hpp"
#include <cassert>
#include <climits>
#include <iostream>
#include <sstream>
#include <thread>

using namespace std;
using namespace nlc;

/// @brief Demonstration sampler class
class SamplerExample {
public:
	/// @brief Constructor of SamplerExample
	/// @param connectedDeviceHandle - the device handle to use
	SamplerExample(Context &menuContext, const DeviceHandle connectedDeviceHandle);
	virtual ~SamplerExample();

	/// @brief Execute all defined example functions
	void process();

	/// @brief Execute all example functions without notification callback
	void processExamplesWithoutNotification();

	/// @brief Execute example function for normal mode without notification callback
	void processSamplerWithoutNotificationNormal();

	/// @brief Execute example function for repetitive mode without notification callback
	void processSamplerWithoutNotificationRepetitive();

	/// @brief Execute example function for continuous mode without notification callback
	void processSamplerWithoutNotificationContinuous();

	/// @brief Execute all example functions with notification callback
	void processExamplesWithNotification();

	/// @brief Execute example function for normal mode with notification callback
	void processSamplerWithNotificationNormal();

	/// @brief Execute example function for repetitive mode with notification callback
	void processSamplerWithNotificationRepetitive();

	/// @brief Execute example function for continuous mode with notification callback
	void processSamplerWithNotificationContinuous();

protected:

	/// @brief Container tor tracked address (name, od index)
	struct TrackedAddress {

		const char *name;

		OdIndex odIndex;
	};

	/// @brief Implementation class of SamplerNotify, handling the notify callback
	class SamplerNotifyExample : public SamplerNotify {
	public:
		/// @brief Constructor of SamplerNotifyExample
		/// @param samplerExample - the sampler example to use
		SamplerNotifyExample(SamplerExample &samplerExample);

		/// @brief Destructor of SamplerNotifyExample
		~SamplerNotifyExample();

		/// @brief Checks if sampler is running
		/// @return - returns true if running, false otherwise
		bool isRunning() const noexcept {
			return samplerRunning;
		}

		/// @brief Deactivate sampler
		void setInactive() noexcept {
			samplerRunning = false;
		}

		/// @brief SamplerNotify callback function
		/// @param lastError - last error received
		/// @param samplerState - the state of the sampler
		/// @param sampleDatas - vector of SampleData (from device)
		/// @param applicationData - data from application for device (not used)
		void notify(const ResultVoid &lastError, const SamplerState samplerState,
					const std::vector<SampleData> &sampleDatas,
					int64_t applicationData) override;

	private:
		SamplerExample &samplerExample;
		volatile bool samplerRunning;		
	};

protected:
	/// @brief Function used for sampler configuration
	/// @param mode - the mode to use (normal, repetitive, continuous)
	void configure(const SamplerMode mode);

	/// @brief Function to start a sampler
	/// @param samplerNotify - callback for notification
	/// @param applicationData - data from application for device (not used)
	void start(SamplerNotifyExample *samplerNotify = nullptr,
			   int64_t applicationData = 0);

	/// @brief Get the state of the sampler
	/// @return - returns the sampler state
	SamplerState getSamplerState();

	/// @brief Get the sampled data from device buffer
	/// @return - returns a vector with the sampled data
	std::vector<SampleData> getSamplerData();

	/// @brief Error handling
	/// @param lastErrorPtr - pointer to the last error
	void handleSamplerFailed(const ResultVoid *lastErrorPtr = nullptr);

	/// @brief Process and display the sampled data
	/// @param sampleDatas - vector with the sampled data
	void processSampledData(const std::vector<SampleData> &sampleDatas);

	static const TrackedAddress trackedAddresses[]; // array containing the addresses to track (max. 12)
	static const OdIndex triggerAddress; // trigger address
	static const SamplerTriggerCondition triggerCondition; // trigger condition
	static const uint32_t triggerValue; // trigger value 
	static const uint32_t triggerValueInactive; // trigger value inactive, depends on trigger condition
	static const uint32_t triggerValueActive; // trigger value active, depends on trigger condition
	static const uint16_t periodMilliseconds; // sample period in milliseconds
	static const uint16_t numberOfSamples; // number of samples
	
	Context &ctx; // menu context
	DeviceHandle deviceHandle; // device handle to use
	volatile uint64_t lastIteration; // last iteration counter
	volatile uint64_t sampleNumber; // sample number
	bool headerPrinted; // output flag for 'table header'
};
