import { routes } from '../router';

export type Routes = (typeof routes)[keyof typeof routes];
