import { Injectable, inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { map } from 'rxjs';
import { AuthService } from '../services/auth.service';

@Injectable({ providedIn: 'root' })
class StartGuardService {
  private auth = inject(AuthService);
  private router = inject(Router);

  can(): ReturnType<CanActivateFn> {
    return this.auth.isLoggedInOnce().pipe(
      map(ok => this.router.createUrlTree([ ok ? '/profile' : '/login' ]))
    );
  }
}

export const StartGuard: CanActivateFn = () => inject(StartGuardService).can();
