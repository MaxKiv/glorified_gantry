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
* @file   sampler_example.cpp
*
* @brief  Definition of sampler example class
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "sampler_example.hpp"

const SamplerExample::TrackedAddress SamplerExample::trackedAddresses[]
	= {{"Up time", OdIndex(0x230F, 0x00)}, {"Temperature", OdIndex(0x4014, 0x03)}};
// In this sample we use first NanoJ input as trigger
const OdIndex SamplerExample::triggerAddress = OdIndex(0x2400, 0x01);
const SamplerTriggerCondition SamplerExample::triggerCondition
	= SamplerTriggerCondition::TC_GREATER;
const uint32_t SamplerExample::triggerValue = 10;
// Depends from condition
const uint32_t SamplerExample::triggerValueInactive = SamplerExample::triggerValue;
// Depends from condition
const uint32_t SamplerExample::triggerValueActive = SamplerExample::triggerValue + 1;
const uint16_t SamplerExample::periodMilliseconds = 1000;
const uint16_t SamplerExample::numberOfSamples = 5;


SamplerExample::SamplerExample(Context &menuContext,
											   const DeviceHandle connectedDeviceHandle)
	: ctx(menuContext), deviceHandle(connectedDeviceHandle) {
}

SamplerExample::~SamplerExample() {
}

SamplerExample::SamplerNotifyExample::SamplerNotifyExample(
	SamplerExample &example)
	: samplerExample(example), samplerRunning(true) {
}

SamplerExample::SamplerNotifyExample::~SamplerNotifyExample() {
	// Destroying this notification object is safe only when the sampler is not active
	assert(!samplerRunning);
}

void SamplerExample::SamplerNotifyExample::notify(
	const ResultVoid &lastError, const SamplerState samplerState,
	const vector<SampleData> &sampleDatas, int64_t applicationData) {

	// Be aware that notifications are executed in the context of separate threads 
	// other than thread that started the sampler.
	// 
	// Be careful when calling Nanolib functionality here, as doing so may cause this method
	// to be called recursively, potentially causing your application to deadlock.
	// 
	// For the same reason, this method should not throw exceptions.

	assert(samplerRunning);

	(void)applicationData;

	if (!sampleDatas.empty())
		samplerExample.processSampledData(sampleDatas);

	if (samplerState == SamplerState::Failed) {
		try {
			samplerExample.handleSamplerFailed(&lastError);
		} catch (...) {
			// see comment above
		}
	}

	if ((samplerState != SamplerState::Ready)
		&& (samplerState != SamplerState::Running)) {
		// It's now safe to destroy this notification object
		samplerRunning = false;
	}
}

void SamplerExample::process() {

	processExamplesWithoutNotification();
	processExamplesWithNotification();
}

void SamplerExample::processExamplesWithoutNotification() {

	processSamplerWithoutNotificationNormal();
	processSamplerWithoutNotificationRepetitive();
	processSamplerWithoutNotificationContinuous();
}

void SamplerExample::processSamplerWithoutNotificationNormal() {

	const chrono::milliseconds sleepTimeMsec(periodMilliseconds);

	cout << "\nSampler without notification in normal mode:" << endl;

	configure(SamplerMode::Normal);
	start();

	SamplerState samplerState;

	do {

		this_thread::sleep_for(sleepTimeMsec);

		processSampledData(getSamplerData());
		samplerState = getSamplerState();

	} while ((samplerState == SamplerState::Ready)
			 || (samplerState == SamplerState::Running));

	// Process any remaining data
	processSampledData(getSamplerData());

	if (samplerState == SamplerState::Failed)
		handleSamplerFailed();
}

void SamplerExample::processSamplerWithoutNotificationRepetitive() {

	const chrono::milliseconds sleepTimeMsec(periodMilliseconds);
	const chrono::milliseconds waitTimeMsec(50);

	cout << "\nSampler without notification in repetative mode:" << endl;

	// configure the sampler
	configure(SamplerMode::Repetitive);
	// start the sampler
	start();

	// wait for sampler to run
	SamplerState samplerState;
	do {
		this_thread::sleep_for(waitTimeMsec);
		samplerState = getSamplerState();
	} while ((samplerState != SamplerState::Running) 
		&& (samplerState != SamplerState::Failed));

	// start processing sampled data
	do {

		this_thread::sleep_for(sleepTimeMsec);

		processSampledData(getSamplerData());
		
		if (lastIteration >= 4) {
			// In repetative mode the sampler will continue to run until it is stopped or an error
			// occurs
			ctx.nanolibAccessor->getSamplerInterface().stop(deviceHandle);
		}

		samplerState = getSamplerState();

	} while ((samplerState == SamplerState::Ready)
			 || (samplerState == SamplerState::Running));

	// Process any remaining data
	processSampledData(getSamplerData());

	if (samplerState == SamplerState::Failed)
		handleSamplerFailed();
}

void SamplerExample::processSamplerWithoutNotificationContinuous() {

	const chrono::milliseconds sleepTimeMsec(periodMilliseconds);
	
	cout << "\nSampler without notification in continuous mode:" << endl;

	configure(SamplerMode::Continuous);
	start();

	SamplerState samplerState(SamplerState::Ready);
	constexpr unsigned maxCycles = 10;
	unsigned cycles = 0;

	do {

		this_thread::sleep_for(sleepTimeMsec);

		processSampledData(getSamplerData());

		if (++cycles == maxCycles) {
			// In continuous mode the sampler will continue to run until it is stopped or an error occurs
			ctx.nanolibAccessor->getSamplerInterface().stop(deviceHandle);
		}

		samplerState = getSamplerState();

	} while ((samplerState == SamplerState::Ready)
			 || (samplerState == SamplerState::Running));

	// Process any remaining data
	processSampledData(getSamplerData());

	if (samplerState == SamplerState::Failed)
		handleSamplerFailed();
}

void SamplerExample::processExamplesWithNotification() {

	processSamplerWithNotificationNormal();
	processSamplerWithNotificationRepetitive();
	processSamplerWithNotificationContinuous();
}

void SamplerExample::processSamplerWithNotificationNormal() {

	const chrono::milliseconds sleepTimeMsec(periodMilliseconds);
		
	cout << "\nSampler with notification in normal mode:" << endl;

	configure(SamplerMode::Normal);

	SamplerNotifyExample samplerNotify(*this);

	start(&samplerNotify);
	while (samplerNotify.isRunning()) {
		this_thread::sleep_for(sleepTimeMsec);
	}
}

void SamplerExample::processSamplerWithNotificationRepetitive() {

	const chrono::milliseconds sleepTimeMsec(periodMilliseconds);
	const chrono::milliseconds waitTimeMsec(50);

	cout << "\nSampler with notification in repetative mode:" << endl;

	configure(SamplerMode::Repetitive);

	SamplerNotifyExample samplerNotify(*this);

	start(&samplerNotify);

	// wait for sampler to run
	SamplerState samplerState;
	do {
		this_thread::sleep_for(waitTimeMsec);
		samplerState = getSamplerState();
	} while ((samplerState != SamplerState::Running) 
		&& (samplerState != SamplerState::Failed));

	// start processing sampled data
	while (samplerNotify.isRunning()) {
		this_thread::sleep_for(sleepTimeMsec);

		if (lastIteration >= 4) {
			// In repetative mode the sampler will continue to run until it is stopped or an error occurs
			ctx.nanolibAccessor->getSamplerInterface().stop(deviceHandle);;
			break;
		}
	}	

	samplerNotify.setInactive();
}

void SamplerExample::processSamplerWithNotificationContinuous() {

	cout << "\nSampler with notification in continuous mode:" << endl;

	configure(SamplerMode::Continuous);

	SamplerNotifyExample samplerNotify(*this);

	start(&samplerNotify);
	this_thread::sleep_for(chrono::milliseconds(periodMilliseconds * 10));
	// In continuous the sampler will continue to run until it is stopped or an error occurs
	ctx.nanolibAccessor->getSamplerInterface().stop(deviceHandle);
}

void SamplerExample::configure(const SamplerMode mode) {

	constexpr size_t numberOfTrackedAddresses
		= sizeof(trackedAddresses) / sizeof(trackedAddresses[0]);

	SamplerConfiguration samplerConfiguration;

	for (size_t trackedAddressIndex = 0; trackedAddressIndex < numberOfTrackedAddresses;
		 ++trackedAddressIndex) {
		samplerConfiguration.trackedAddresses.push_back(
			trackedAddresses[trackedAddressIndex].odIndex);
	}

	// setup start trigger
	SamplerTrigger samplerTriggerStart;
	samplerTriggerStart.condition = triggerCondition;
	samplerTriggerStart.address = triggerAddress;
	samplerTriggerStart.value = triggerValue;

	// set start trigger
	samplerConfiguration.startTrigger = samplerTriggerStart;
	samplerConfiguration.periodMilliseconds = periodMilliseconds;
	// in continuous mode, duration has to be zero
	if (mode == SamplerMode::Continuous) {
		samplerConfiguration.durationMilliseconds = 0;
	} else {
		samplerConfiguration.durationMilliseconds = 4000;
	}
	// Currrently this value is not used
	samplerConfiguration.preTriggerNumberOfSamples = 0;
	samplerConfiguration.mode = mode;
	samplerConfiguration.usingSoftwareImplementation = (mode == SamplerMode::Continuous);

	ctx.nanolibAccessor->getSamplerInterface().configure(deviceHandle, samplerConfiguration);
}

void SamplerExample::start(SamplerNotifyExample *samplerNotify /* = nullptr*/,
								  int64_t applicationData /* = 0*/) {

	lastIteration = 0;
	sampleNumber = 0;
	headerPrinted = false;

	// Deactivate the start trigger
	ctx.nanolibAccessor->writeNumber(deviceHandle, triggerValueInactive, triggerAddress, 32);

	// Start the sampler
	try {
		ctx.nanolibAccessor->getSamplerInterface().start(deviceHandle, samplerNotify, applicationData);
	} catch (...) {
		if (samplerNotify != nullptr)
			samplerNotify->setInactive();
		throw;
	}

	// Activate start trigger
	try {
		ctx.nanolibAccessor->writeNumber(deviceHandle, triggerValueActive, triggerAddress, 32);
	} catch (...) {
		ctx.nanolibAccessor->getSamplerInterface().stop(deviceHandle);
		throw;
	}
}

SamplerState SamplerExample::getSamplerState() {
	return ctx.nanolibAccessor->getSamplerInterface().getState(deviceHandle).getResult();
}

vector<SampleData> SamplerExample::getSamplerData() {
	return ctx.nanolibAccessor->getSamplerInterface().getData(deviceHandle).getResult();
}

void SamplerExample::handleSamplerFailed(
	const ResultVoid *lastErrorPtr /* = nullptr*/) {

	ResultVoid lastError;

	if (lastErrorPtr != nullptr) {
		lastError = *lastErrorPtr;
	} else {
		assert(getSamplerState() == SamplerState::Failed);
		lastError = ctx.nanolibAccessor->getSamplerInterface().getLastError(deviceHandle);
	}

	assert(lastError.hasError());
	cerr << endl
			  << "Sampler execution failed with error: " << lastError.getError() << endl;
}

void SamplerExample::processSampledData(const vector<SampleData> &sampleDatas) {

	constexpr size_t numberOfTrackedAddresses
		= sizeof(trackedAddresses) / sizeof(trackedAddresses[0]);

	for (const auto sampleData : sampleDatas) {

		const auto &sampledValues = sampleData.sampledValues;
		const size_t numberOfSampledValues = sampledValues.size();

		assert((numberOfSampledValues % numberOfTrackedAddresses) == 0);

		if (lastIteration != sampleData.iterationNumber) {
			sampleNumber = 0;
			lastIteration = sampleData.iterationNumber;
		}
		
		stringstream stream;

		if (!headerPrinted) {

			static const char *cszHorzLine = "------------------------------------------------------------\n";

			stream << cszHorzLine;
			stream << left << setw(10) << "Iteration" << left << setw(10)
				   << "Sample";
			for (size_t trackedAddressIndex = 0; trackedAddressIndex < numberOfTrackedAddresses;
				 ++trackedAddressIndex) {

				string addressName("[");

				addressName += trackedAddresses[trackedAddressIndex].name;
				addressName += "]";
				stream << left << setw(14) << addressName << left << setw(8)
					   << "Time";
			}

			stream << "\n";
			stream << cszHorzLine;

			headerPrinted = true;
		}

		for (size_t index = 0; index < numberOfSampledValues; index += numberOfTrackedAddresses) {

			stream << left << setw(10) << lastIteration;
			stream << left << setw(10) << sampleNumber;

			for (size_t trackedAddressIndex = 0; trackedAddressIndex < numberOfTrackedAddresses;
				 ++trackedAddressIndex) {

				const auto &sampledValue = sampledValues[index + trackedAddressIndex];

				stream << left << setw(14) << sampledValue.value;
				stream << left << setw(8) << sampledValue.collectTimeMsec;
			}
			
			stream << "\n";
			++sampleNumber;
		}

		cout << stream.str();
	}
}