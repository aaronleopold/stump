import API, { baseURL } from '..';

export function getSeriesById(id: string): Promise<ApiResult<Series>> {
	return API.get(`/series/${id}`);
}

export function getSeriesMedia(id: string, page: number): Promise<PageableApiResult<Media[]>> {
	return API.get(`/series/${id}/media?page=${page}`);
}

export function getSeriesThumbnail(id: string): string {
	return `${baseURL}/series/${id}/thumbnail`;
}
