import { ApiResult, LogFileMeta } from '@stump/core';
import API from '..';

export function getLogFileMeta(): Promise<ApiResult<LogFileMeta>> {
	return API.get('/logs');
}
