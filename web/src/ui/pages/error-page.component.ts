import { ChangeDetectionStrategy, Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-error-page',
  standalone: true,
  imports: [RouterLink],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <section class="mt-10 text-center">
      <h1 class="text-3xl font-bold mb-2">Something went wrong</h1>
      <p class="text-gray-700 mb-6">We couldn't find the page or an unexpected error occurred.</p>
      <a routerLink="/" class="px-4 py-2 bg-gray-800 text-white rounded">Go Home</a>
    </section>
  `
})
export class ErrorPageComponent {}
