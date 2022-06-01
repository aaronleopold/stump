import axios from 'axios';

export const baseURL = import.meta.env.PROD
	? `${import.meta.env.BASE_URL}api`
	: 'http://localhost:81001/api';

console.log(baseURL);

const API = axios.create({
	baseURL,
	withCredentials: true,
});

export default API;
