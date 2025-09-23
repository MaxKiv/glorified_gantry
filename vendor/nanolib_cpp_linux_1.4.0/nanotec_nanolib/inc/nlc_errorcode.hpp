/** \file nlc_errorcode.hpp
 * NlcErrorCode enumeration class
 */
#pragma once

/**
@brief Namespace nlc
*/
namespace nlc {

/** @enum NlcErrorCode
 * @brief Nanolib error codes
 */
enum class NlcErrorCode {

	/**
	 * \code
	 * Category:    none
	 * Description: No error
	 * Reason:      The operation completed successfully.
	 * \endcode
	 */
	Success = 0,

	/**
	 * \code
	 * Category:    Unspecified
	 * Description: Unspecified error
	 * Reason:      Failure that cannot be assigned to other categories.
	 * \endcode
	 */
	GeneralError = 1,

	/**
	 * \code
	 * Category:     Bus
	 * Description:  The hardware bus is not available
	 * Reason:       The hardware bus does not exist or no longer available.
	 * \endcode
	 */
	BusUnavailable = 100,

	/**
	 * \code
	 * Category:     Communication
	 * Description:  Unreliable communication
	 * Reason:       Unexpected data, wrong CRC, frame or parity errors, etc.
	 * \endcode
	 */
	CommunicationError = 200,

	/**
	 * \code
	 * Category:     Protocol
	 * Description:  Protocol error
	 * Reason:       Response following unsupported protocol option, device report unsupported
	 * protocol, error in the protocol (e.g. SDO segement sync bit), etc. 
	 * \endcode
	 */
	ProtocolError = 300,

	/**
	 * \code
	 * Category:     Object Dictionary
	 * Description:  The OD address does not exist
	 * Reason:       The address does not exist in the Object Dictionary.
	 * \endcode
	 */
	ODDoesNotExist = 400,

	/**
	 * \code
	 * Category:     Object Dictionary
	 * Description:  Invalid access to OD address
	 * Reason:       Attempt to write to a read-only address, attempt to read from a write-only
	 * address. 
	 * \endcode
	 */
	ODInvalidAccess,

	/**
	 * \code
	 * Category:     Object Dictionary
	 * Description:  Type Mismatch
	 * Reason:       The value cannot be converted to the specified type.
	 *               For example - an attempt to treat a string as a number.
	 * \endcode
	 */
	ODTypeMismatch,

	/**
	 * \code
	 * Category:     Application
	 * Description:  Operation has been aborted
	 * Reason:       The operation has been aborted by a request from the application.
	 *               Returns only upon interrupt of operation from callback function (e.g. bus
	 * scanning) 
	 * \endcode
	 */
	OperationAborted = 500,

	/**
	 * \code
	 * Category:     Common
	 * Description:  Operation not supported
	 * Reason:       The operation is not supported on the hardware bus or device.
	 * \endcode
	 */
	OperationNotSupported = 600,

	/**
	 * \code
	 * Category:     Common
	 * Description:  Incorrect or invalid operation
	 * Reason:       The requested operation is incorrect in the current context or invalid with the
	 * current arguments. Attempt to reconnect to an already connected bus or device. Attempt to
	 * disconnect from a bus or a device that has already been disconnected. Attempt to perform
	 * bootloader operation in firmware mode or vice versa. 
	 * \endcode
	 */
	InvalidOperation,

	/**
	 * \code
	 * Category:     Common
	 * Description:  Invalid arguments
	 * Reason:       The arguments passed are invalid.
	 * \endcode
	 */
	InvalidArguments,

	/**
	 * \code
	 * Category:     Common
	 * Description:  Access is denied
	 * Reason:       The current execution context does not have sufficient privileges or
	 * capabilities to perform the requested operation. 
	 * \endcode
	 */
	AccessDenied,

	/**
	 * \code
	 * Category:     Common
	 * Description:  The specified resource was not found
	 * Reason:       The specified hardware bus, protocol, device, OD address on device, or file was
	 * not found. 
	 * \endcode
	 */
	ResourceNotFound,

	/**
	 * \code
	 * Category:     Common
	 * Description:  The specified resource is not available
	 * Reason:       The specified resource does not exist or is temporarily partially or completely
	 * unavailable. 
	 * \endcode
	 */
	ResourceUnavailable,

	/**
	 * \code
	 * Category:     Common
	 * Description:  Not enough memory
	 * Reason:       Not enough memory resources are available to process this command.
	 * \endcode
	 */
	OutOfMemory,

	/**
	 * \code
	 * Category:     Common
	 * Description:  Operation timed out
	 * Reason:       The operation returned because the timeout period expired.
	 *               The timeout may be a device response time, a time to gain shared or exclusive
	 * access to a resource, or a time to switch the bus or device to a suitable state. 
	 * \endcode
	 */
	TimeoutError
};

} // namespace nlc
