import { Injectable, inject } from '@angular/core';
import { CanActivateFn, Router, UrlTree } from '@angular/router';
import { map } from 'rxjs';
import { AuthService } from '../services/auth.service';

@Injectable({ providedIn: 'root' })
class AuthGuardService {
  private auth = inject(AuthService);
  private router = inject(Router);

  can(): ReturnType<CanActivateFn> {
    return this.auth.isLoggedInOnce().pipe(
      map(ok => ok ? true : this.router.createUrlTree(['/login']))
    );
  }
}

export const AuthGuard: CanActivateFn = () => inject(AuthGuardService).can();
