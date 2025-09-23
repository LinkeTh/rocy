import { Injectable, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { map, catchError, of, tap } from 'rxjs';
import { environment } from '../../environments/environment';

export interface MeResponse {
  id?: string | number;
  email?: string;
  name?: string;
  picture?: string;
  [k: string]: unknown;
}

@Injectable({ providedIn: 'root' })
export class AuthService {
  private http = inject(HttpClient);
  readonly user = signal<MeResponse | null>(null);
  readonly loading = signal<boolean>(false);

  get apiBase() { return environment.apiBase; }

  me() {
    this.loading.set(true);
    return this.http.get<MeResponse>(`${this.apiBase}/me`, { withCredentials: true }).pipe(
      tap(user => this.user.set(user)),
      map(user => !!user && Object.keys(user).length > 0),
      catchError(_ => {
        this.user.set(null);
        return of(false);
      }),
      tap(_ => this.loading.set(false))
    );
  }

  isLoggedInOnce() {
    // Convenience method to check once; subscribes immediately and completes.
    return this.me();
  }

  login() {
    window.location.href = `${this.apiBase}/login`;
  }

  logout() {
    window.location.href = `${this.apiBase}/logout`;
  }
}
