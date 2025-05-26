import { PostgrestError } from '@supabase/supabase-js';
import { ApiErrorResponse } from '../types/api.types';

/**
 * Standard error codes for the API
 */
export enum ErrorCode {
    UNAUTHORIZED = 'unauthorized',
    FORBIDDEN = 'forbidden',
    NOT_FOUND = 'not_found',
    BAD_REQUEST = 'bad_request',
    CONFLICT = 'conflict',
    INTERNAL_ERROR = 'internal_error',
    VALIDATION_ERROR = 'validation_error',
    }

    /**
     * Create a standard API error response
     * @param message Error message
     * @param code Error code
     * @param details Additional error details
     * @returns API error response object
     */
    export function createErrorResponse(
    message: string,
    code: ErrorCode = ErrorCode.INTERNAL_ERROR,
    details?: Record<string, unknown>
    ): ApiErrorResponse {
    return {
        error: message,
        code,
        details,
    };
    }

    /**
     * Handle Supabase errors and convert them to standard API error responses
     * @param error PostgrestError from Supabase
     * @returns API error response object
     */
    export function handleSupabaseError(error: PostgrestError): ApiErrorResponse {
    // Map Supabase error codes to our standard codes
    let code = ErrorCode.INTERNAL_ERROR;
    
    if (error.code === '23505') {
        // Unique violation
        code = ErrorCode.CONFLICT;
    } else if (error.code === '42P01') {
        // Undefined table
        code = ErrorCode.INTERNAL_ERROR;
    } else if (error.code === '42703') {
        // Undefined column
        code = ErrorCode.INTERNAL_ERROR;
    } else if (error.code === '23503') {
        // Foreign key violation
        code = ErrorCode.BAD_REQUEST;
    } else if (error.code === '23502') {
        // Not null violation
        code = ErrorCode.VALIDATION_ERROR;
    } else if (error.code === '22P02') {
        // Invalid text representation
        code = ErrorCode.VALIDATION_ERROR;
    } else if (error.code?.startsWith('28')) {
        // Authorization errors
        code = ErrorCode.UNAUTHORIZED;
    } else if (error.code?.startsWith('42')) {
        // Syntax or name errors
        code = ErrorCode.BAD_REQUEST;
    }

    return {
        error: error.message || 'An error occurred while processing your request',
        code,
        details: {
        hint: error.hint,
        code: error.code,
        },
    };
}

    /**
     * Handle generic errors and convert them to standard API error responses
     * @param error Any error object
     * @returns API error response object
     */
    export function handleGenericError(error: unknown): ApiErrorResponse {
    if (error instanceof PostgrestError) {
        return handleSupabaseError(error);
    }

    const message = (error as { message?: string })?.message || 'An unexpected error occurred';
    
    return {
        error: message,
        code: ErrorCode.INTERNAL_ERROR,
        details: process.env.NODE_ENV === 'development' ? error : undefined,
    };
    }

    /**
     * Create a not found error response
     * @param resource The resource that was not found
     * @returns API error response object
     */
    export function notFoundError(resource: string): ApiErrorResponse {
    return createErrorResponse(
        `The requested ${resource} was not found`,
        ErrorCode.NOT_FOUND
    );
    }

    /**
     * Create an unauthorized error response
     * @returns API error response object
     */
    export function unauthorizedError(): ApiErrorResponse {
    return createErrorResponse(
        'You must be logged in to perform this action',
        ErrorCode.UNAUTHORIZED
    );
    }

    /**
     * Create a forbidden error response
     * @returns API error response object
     */
    export function forbiddenError(): ApiErrorResponse {
    return createErrorResponse(
        'You do not have permission to perform this action',
        ErrorCode.FORBIDDEN
    );
    }

    /**
     * Create a validation error response
     * @param message Error message
     * @param details Validation error details
     * @returns API error response object
     */
    export function validationError(message: string, details?: Record<string, unknown>): ApiErrorResponse {
    return createErrorResponse(
        message,
        ErrorCode.VALIDATION_ERROR,
        details
    );
}