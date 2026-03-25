import type { ThanksMember } from '@/types/thanks';
import { honoClient } from './honoClient';

export interface ThanksRepository {
  getSpecialThanks(): Promise<ThanksMember[]>;
}

class ApiThanksRepository implements ThanksRepository {
  async getSpecialThanks(): Promise<ThanksMember[]> {
    return honoClient.getSpecialThanks();
  }
}

export const thanksRepository: ThanksRepository = new ApiThanksRepository();
