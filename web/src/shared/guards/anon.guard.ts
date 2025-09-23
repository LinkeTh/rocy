import { Injectable, inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { map } from 'rxjs';
import { AuthService } from '../services/auth.service';

@Injectable({ providedIn: 'root' })
class AnonGuardService {
  private auth = inject(AuthService);
  private router = inject(Router);

  can(): ReturnType<CanActivateFn> {
    return this.auth.isLoggedInOnce().pipe(
      map(ok => ok ? this.router.createUrlTree(['/profile']) : true)
    );
  }
}

export const AnonGuard: CanActivateFn = () => inject(AnonGuardService).can();
