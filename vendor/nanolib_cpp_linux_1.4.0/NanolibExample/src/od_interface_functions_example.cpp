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
* @file   od_interface_functions_example.cpp
*
* @brief  Definition of object dictionary interface specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "od_interface_functions_example.hpp"

void readNumber(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "Reading mode of operation (" << odModeOfOperation.toString() << ") ..." << endl;
	ResultInt resultInt = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odModeOfOperation);
	if (resultInt.hasError()) {
        handleErrorMessage(ctx, "Error during readNumber: ", resultInt.getError());
	}
	cout << odModeOfOperation.toString() << " = " << resultInt.getResult() << endl;
	cout << "This is only the raw value. The OD value might be signed or unsigned up to a total length of 4 bytes" << endl << endl;

	cout << "Reading SI unit position (" << odSIUnitPosition.toString() << ") ... " << endl;
	resultInt = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odSIUnitPosition);
	if (resultInt.hasError()) {
        handleErrorMessage(ctx,  "Error during readNumber: ", resultInt.getError());
		return;
	}
	cout << odSIUnitPosition.toString() << " = " << resultInt.getResult() << endl;
	cout << "This is only the raw value. The OD value might be signed or unsigned up to a total length of 4 bytes" << endl;
}

void readString(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "Reading Nanotec home page string (" << odHomePage.toString() << ") ..." << endl;
	ResultString resultString = ctx.nanolibAccessor->readString(ctx.activeDevice, odHomePage);
	if (resultString.hasError()) {
        handleErrorMessage(ctx, "Error during readString: ", resultString.getError());
	}
	cout << odHomePage.toString() << " = '" << resultString.getResult() << "'" << endl;
}

void readArray(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx,  "No active device set. Select an active device first.");
		return;
	} 

	cout << "Reading device error stack (0x1003) ..." << endl;
	ResultArrayInt resultArrayInt = ctx.nanolibAccessor->readNumberArray(ctx.activeDevice, odErrorStackIndex);
	if (resultArrayInt.hasError()) {
        handleErrorMessage(ctx, "Error during readArray: ", resultArrayInt.getError());
	}

	// output only the first field (error count)
	// fields with index > 0 would contain specific stored errors
	vector<int64_t> errorStack = resultArrayInt.getResult();
	cout << "The error stack has " << to_string(errorStack.size()) << " elements" << endl;
	cout << "The first element (error count) is: " << to_string(errorStack.at(0)) << endl;
}

void writeNumber(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "Writing motor stop command to control word (" << odControlWord.toString() << " = 0x06) ..." << endl;
	ResultVoid resultVoid = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 6, odControlWord, 16);
	if (resultVoid.hasError()) {
        handleErrorMessage(ctx, "Error during writeNumber: ", resultVoid.getError());
	}
}

void assignObjectDictionary(Context &ctx) {
	ctx.waitForUserConfirmation = false;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	optional<string> inputPath;
	cout << "Please enter the directory (path) where the od.xml is located: ";
	do {
		inputPath = getline(cin);
	} while (!inputPath.has_value());

	ResultObjectDictionary resultObjectDictionary = ctx.nanolibAccessor->autoAssignObjectDictionary(
		ctx.activeDevice, inputPath.value());

	if (resultObjectDictionary.hasError()) {
        handleErrorMessage(ctx, "Error during assignObjectDictionary: ", resultObjectDictionary.getError());
		return;
	}
}

void readNumberViaDictionaryInterface(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultObjectDictionary resultObjectDictionary = ctx.nanolibAccessor->getAssignedObjectDictionary(ctx.activeDevice);
	if (resultObjectDictionary.hasError()) {
        handleErrorMessage(ctx, "Error during readNumberViaDictionaryInterface: ", resultObjectDictionary.getError());
		return;
	}

	if (resultObjectDictionary.getResult().getXmlFileName().getResult().empty()) {
		cout << ctx.light_yellow << "No valid object dictionary assigned. Using fallback method!" << ctx.def << endl;
	} 

	ObjectDictionary &objectDictionary = const_cast<ObjectDictionary&>(resultObjectDictionary.getResult());

	if (objectDictionary.getDeviceHandle().getResult().get() != ctx.activeDevice.get()) {
        handleErrorMessage(ctx, "", "Object dictionary mismatch in readNumberViaDictionaryInterface.");
		return;
	} 
 
	cout << "Reading mode of operation (" << odModeOfOperation.toString() << ") ..." << endl;
	ResultInt resultInt = objectDictionary.readNumber(odModeOfOperation);
	if (resultInt.hasError()) {
        handleErrorMessage(ctx, "Error during readNumberViaDictionaryInterface: ", resultInt.getError());
	}
	// OD 0x6060:00 is of type int8_t so the user has to cast accordingly
	int8_t modeOfOperation = static_cast<int8_t>(resultInt.getResult());
	cout << odModeOfOperation.toString() << " = " << to_string(modeOfOperation) << endl;

	cout << "Some object entry properties: " << endl;
	const ObjectEntry &objectEntry = objectDictionary.getObjectEntry(odModeOfOperation.getIndex()).getResult();
	cout << "Object(" << odModeOfOperation.toString() << ").ObjectCode = ";
	string objectCodeString = "";
	switch (objectEntry.getObjectCode()) {
		case ObjectCode::Null:
			objectCodeString = "Null";
			break;
		case ObjectCode::Deftype:
			objectCodeString = "Deftype";
			break;
		case ObjectCode::Defstruct:
			objectCodeString = "Defstruct";
			break;
		case ObjectCode::Var:
			objectCodeString = "Var";
			break;
		case ObjectCode::Array:
			objectCodeString = "Array";
			break;
		case ObjectCode::Record:
			objectCodeString = "Record";
			break;
		default:
			objectCodeString = to_string(static_cast<int>(objectEntry.getObjectCode()));
			break;
	}
	cout << objectCodeString << endl;

	cout << "Object(0x6060).DataType = " 
					  << OdTypesHelper::objectEntryDataTypeToString(objectEntry.getDataType()) 
					  << endl;
	
	cout << "Some ObjectSubEntry properties: " << endl;
	const ObjectSubEntry &objectSubEntry = objectDictionary.getObject(odModeOfOperation).getResult();
	cout << "OdIndex(" << odModeOfOperation.toString() << ").DataType = " 
					   << OdTypesHelper::objectEntryDataTypeToString(objectSubEntry.getDataType()) 
					   << endl;
	cout << "OdIndex(" << odModeOfOperation.toString() << ").BitLength = " 
					   << to_string(objectSubEntry.getBitLength()) << endl << endl;


	cout << "Reading SI unit position (" << odSIUnitPosition.toString() << ") ... " << endl;
	resultInt = objectDictionary.readNumber(odSIUnitPosition);
	if (resultInt.hasError()) {
        handleErrorMessage(ctx,  "Error during readNumberViaDictionaryInterface: ", resultInt.getError());
		return;
	}
	// OD 0x60A8:00 is of type uint32_t so the user has to cast accordingly
	uint32_t unitPosition = static_cast<uint32_t>(resultInt.getResult());
	cout << odSIUnitPosition.toString() << " = " << to_string(unitPosition) << endl;

	cout << "Some object entry properties: " << endl;
	const ObjectEntry &objectEntry2 = objectDictionary.getObjectEntry(odSIUnitPosition.getIndex()).getResult();
	cout << "Object(" << odSIUnitPosition.toString() << ").ObjectCode = ";
	objectCodeString = "";
	switch (objectEntry2.getObjectCode()) {
		case ObjectCode::Null:
			objectCodeString = "Null";
			break;
		case ObjectCode::Deftype:
			objectCodeString = "Deftype";
			break;
		case ObjectCode::Defstruct:
			objectCodeString = "Defstruct";
			break;
		case ObjectCode::Var:
			objectCodeString = "Var";
			break;
		case ObjectCode::Array:
			objectCodeString = "Array";
			break;
		case ObjectCode::Record:
			objectCodeString = "Record";
			break;
		default:
			objectCodeString = to_string(static_cast<int>(objectEntry2.getObjectCode()));
			break;
	}
	cout << objectCodeString << endl;

	cout << "Object(0x60A8).DataType = " 
					  << OdTypesHelper::objectEntryDataTypeToString(objectEntry2.getDataType()) 
					  << endl;
	
	cout << "Some ObjectSubEntry properties: " << endl;
	const ObjectSubEntry &objectSubEntry2 = objectDictionary.getObject(odSIUnitPosition).getResult();
	cout << "OdIndex(" << odSIUnitPosition.toString() << ").DataType = " 
					   << OdTypesHelper::objectEntryDataTypeToString(objectSubEntry2.getDataType()) 
					   << endl;
	cout << "OdIndex(" << odSIUnitPosition.toString() << ").BitLength = " 
					   << to_string(objectSubEntry2.getBitLength()) << endl;
}

void writeNumberViaDictionaryInterface(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultObjectDictionary resultObjectDictionary = ctx.nanolibAccessor->getAssignedObjectDictionary(ctx.activeDevice);
	if (resultObjectDictionary.hasError()) {
        handleErrorMessage(ctx, "Error during writeNumberViaDictionaryInterface: ", resultObjectDictionary.getError());
		return;
	}

	if (resultObjectDictionary.getResult().getXmlFileName().getResult().empty()) {
		cout << ctx.light_yellow << "No valid object dictionary assigned. Using fallback method!" << ctx.def << endl;
	} 

	ObjectDictionary &objectDictionary = const_cast<ObjectDictionary&>(resultObjectDictionary.getResult());

	if (objectDictionary.getDeviceHandle().getResult().get() != ctx.activeDevice.get()) {
        handleErrorMessage(ctx, "", "Object dictionary mismatch in writeNumberViaDictionaryInterface.");
		return;
	} 

	cout << "Writing motor stop command to control word (" << odControlWord.toString() << ") with value 0x06 ..." << endl;
	int64_t value = 6;
	ResultVoid writeResult = objectDictionary.writeNumber(odControlWord, value);

	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during writeNumberViaDictionaryInterface: ", writeResult.getError());
		return;
	}
}
