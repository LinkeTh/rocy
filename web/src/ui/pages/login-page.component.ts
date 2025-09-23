import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { AuthService } from '../../shared/services/auth.service';

@Component({
  selector: 'app-login-page',
  standalone: true,
  imports: [],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <section class="mt-10">
      <h1 class="text-2xl font-bold mb-4">Welcome</h1>
      <p class="mb-6">Please sign in to continue.</p>
      <button (click)="login()" class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">Login with Google</button>
    </section>
  `
})
export class LoginPageComponent {
  private auth = inject(AuthService);
  login() {
    this.auth.login();
  }
}
